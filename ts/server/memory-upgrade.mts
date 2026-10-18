// The explicit persisted-memory upgrade and the `memory upgrade-status` /
// `memory migrate` CLI of the JavaScript server: the twin of
// rust/src/memory/upgrade.rs (`migrate_memory`, `migrate_memory_with_pre_commit`,
// `migrate_v1_to_v2`, `write_verified_backup`, `default_backup_path`,
// `default_receipt_path`, `validate_auxiliary_paths`, `temporary_path`,
// `noop_receipt`) and rust/src/cli_memory.rs (`run_memory`'s
// `UpgradeStatus` and `Migrate` arms).
//
// The transaction holds the same advisory writer lock every Rust writer takes
// (`fs2` `flock` on `<memory>.lock`). Node has no `flock`, so a helper process
// (perl's `flock`, the same system call) holds it for the whole transaction
// and releases it on exit; when no helper can run, the migration fails closed
// with `memory_locked`. Status and refusal wording is data
// (data/meta/server-messages.lino).

import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

import {
  ROOT_HEADER, TARGET_SCHEMA, V1_TO_V2_MIGRATION_ID, inspectMemoryUpgrade, preflightMemoryUpgrade, sharedMemoryPath,
} from './memory-store.mjs';
import { serverMessage } from './messages.mjs';
import { packageVersion } from './seed.mjs';

const SCHEMA_MARKER = 'schema_version';
let tempSequence = 0;

const sha256Hex = (bytes) => createHash('sha256').update(bytes).digest('hex');

/** A `MemoryUpgradeError`: `code`, `message`, and the status when one was read. */
export class MemoryUpgradeError extends Error {
  constructor(code, message, status = null) {
    super(message);
    this.code = code;
    this.status = status;
  }
}

/** Mirrors `memory_lock_path`. */
export const memoryLockPath = (file) => path.join(path.dirname(file), `${path.basename(file) || 'memory.lino'}.lock`);

/** Mirrors `default_backup_path`: content-addressed by the source digest. */
export const defaultBackupPath = (file, digest) => path.join(path.dirname(file), `${path.basename(file) || 'memory.lino'}.schema-1.${digest.slice(0, 12)}.backup`);

/** Mirrors `default_receipt_path`. */
export const defaultReceiptPath = (file) => path.join(path.dirname(file), `${path.basename(file) || 'memory.lino'}.upgrade-receipt.json`);

/** Mirrors `temporary_path`. */
function temporaryPath(file, purpose) {
  tempSequence += 1;
  return path.join(path.dirname(file), `.${path.basename(file) || 'memory.lino'}.${purpose}.${process.pid}.${tempSequence - 1}`);
}

const LOCK_HELPER = 'use Fcntl qw(:flock); open(my $f, ">>", $ARGV[0]) or die "$!\\n"; '
  + 'flock($f, LOCK_EX | LOCK_NB) or die "$!\\n"; $| = 1; print "locked\\n"; <STDIN>;';

/**
 * Hold an exclusive non-blocking `flock` on `lockPath` (the `fs2`
 * `try_lock_exclusive` every Rust writer contends on) until `release()`.
 * Rejects with the helper's error when the lock is held elsewhere.
 * @returns {Promise<{release: () => Promise<void>}>}
 */
export function holdExclusiveLock(lockPath) {
  return new Promise((resolve, reject) => {
    const child = spawn('perl', ['-e', LOCK_HELPER, lockPath], { stdio: ['pipe', 'pipe', 'pipe'] });
    let stdout = '';
    let stderr = '';
    let settled = false;
    const exited = new Promise((done) => child.once('close', done));
    child.stderr.on('data', (chunk) => {
      stderr += chunk;
    });
    child.stdout.on('data', (chunk) => {
      stdout += chunk;
      if (!settled && stdout.includes('locked\n')) {
        settled = true;
        resolve({
          async release() {
            child.stdin.end();
            await exited;
          },
        });
      }
    });
    child.once('error', (error) => {
      if (settled) return;
      settled = true;
      reject(error);
    });
    child.once('close', (code) => {
      if (settled) return;
      settled = true;
      reject(new Error(stderr.trim() || `lock_helper_exit:${code}`));
    });
  });
}

/** Mirrors `write_new_file`: create-new, permissions, write, `sync_all`. */
function writeNewFile(file, bytes, mode = null) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  const descriptor = fs.openSync(file, 'wx');
  try {
    if (mode !== null) fs.fchmodSync(descriptor, mode);
    fs.writeSync(descriptor, bytes);
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
}

/** Mirrors the Unix `sync_parent`. */
function syncParent(file) {
  if (process.platform === 'win32') return;
  const descriptor = fs.openSync(path.dirname(file) || '.', 'r');
  try {
    fs.fsyncSync(descriptor);
  } finally {
    fs.closeSync(descriptor);
  }
}

/** Mirrors `write_atomic_with_permissions`. */
function writeAtomicWithMode(file, bytes, mode) {
  const staged = temporaryPath(file, 'atomic');
  writeNewFile(staged, bytes, mode);
  try {
    fs.renameSync(staged, file);
  } catch (error) {
    fs.rmSync(staged, { force: true });
    throw error;
  }
  syncParent(file);
}

/** Mirrors `write_verified_backup`: reuse an identical backup, refuse a different one, verify a new one. */
function writeVerifiedBackup(file, original, digest, mode) {
  if (fs.existsSync(file)) {
    let existing;
    try {
      existing = fs.readFileSync(file);
    } catch (error) {
      throw new MemoryUpgradeError('backup_read_failed', `backup_read_failed:path=${file}:error=${error.message}`);
    }
    if (sha256Hex(existing) !== digest || !existing.equals(original)) {
      throw new MemoryUpgradeError('backup_conflict', `backup_conflict:path=${file}`);
    }
    return;
  }
  try {
    writeAtomicWithMode(file, original, mode);
  } catch (error) {
    throw new MemoryUpgradeError('backup_write_failed', `backup_write_failed:path=${file}:error=${error.message}`);
  }
  let verified;
  try {
    verified = fs.readFileSync(file);
  } catch (error) {
    throw new MemoryUpgradeError('backup_read_failed', `backup_read_failed:path=${file}:error=${error.message}`);
  }
  if (!verified.equals(original) || sha256Hex(verified) !== digest) {
    throw new MemoryUpgradeError('backup_verification_failed', `backup_verification_failed:path=${file}`);
  }
}

/** Mirrors `migrate_v1_to_v2`: insert only the additive root marker. */
export function migrateV1ToV2(original) {
  const marker = `  ${SCHEMA_MARKER} "${TARGET_SCHEMA}"`;
  if (original.length === 0) return Buffer.from(`${ROOT_HEADER}\n${marker}\n`);
  let text;
  try {
    text = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(original);
  } catch {
    throw new MemoryUpgradeError('memory_not_utf8', serverMessage('memory_upgrade_migrated_not_utf8'));
  }
  if (!text.startsWith(ROOT_HEADER)) {
    throw new MemoryUpgradeError('memory_header_unknown', serverMessage('memory_upgrade_header_unrecognized'));
  }
  const suffix = text.slice(ROOT_HEADER.length);
  let newline;
  let remainder;
  if (suffix.startsWith('\r\n')) [newline, remainder] = ['\r\n', suffix.slice(2)];
  else if (suffix.startsWith('\n')) [newline, remainder] = ['\n', suffix.slice(1)];
  else if (suffix === '') [newline, remainder] = ['\n', ''];
  else throw new MemoryUpgradeError('memory_header_invalid', serverMessage('memory_upgrade_header_own_line'));
  return Buffer.from(`${ROOT_HEADER}${newline}${marker}${newline}${remainder}`);
}

/** Mirrors `noop_receipt`. */
function noopReceipt(file, status) {
  return {
    binary_version: packageVersion(),
    changed: false,
    migration_id: null,
    from_schema_version: status.detected_schema_version,
    to_schema_version: status.detected_schema_version ?? TARGET_SCHEMA,
    memory_path: file,
    backup_path: null,
    receipt_path: null,
    original_sha256: status.source_sha256,
    migrated_sha256: status.source_sha256,
    event_count: status.event_count ?? 0,
    rollback_supported: true,
    rollback_strategy: 'not_required',
  };
}

/** Rust `serde_json::to_vec_pretty`. */
export const prettyJson = (value) => JSON.stringify(value, null, 2);

/**
 * Mirrors `migrate_memory_with_pre_commit`: the whole transaction under the
 * shared writer lock. `beforeCommit(stagedPath)` may throw to simulate an
 * interruption: the staged files are removed and the original stays
 * byte-identical, with the verified backup kept for the retry.
 * @returns {Promise<object>} the `MemoryMigrationReceipt`
 */
export async function migrateMemory(file, backupPath = null, receiptPath = null, beforeCommit = () => {}) {
  if (file === '-') throw new MemoryUpgradeError('memory_path_invalid', serverMessage('memory_upgrade_path_invalid'));
  const lockPath = memoryLockPath(file);
  try {
    fs.mkdirSync(path.dirname(lockPath), { recursive: true });
    fs.closeSync(fs.openSync(lockPath, 'a'));
  } catch (error) {
    throw new MemoryUpgradeError('lock_open_failed', `lock_open_failed:path=${lockPath}:error=${error.message}`);
  }
  let lock;
  try {
    lock = await holdExclusiveLock(lockPath);
  } catch (error) {
    throw new MemoryUpgradeError('memory_locked', `memory_locked:path=${file}:lock=${lockPath}:error=${error.message}`, preflightMemoryUpgrade(file));
  }
  try {
    return transaction(file, backupPath, receiptPath, beforeCommit, lockPath);
  } finally {
    await lock.release();
  }
}

function transaction(file, backupPath, receiptPath, beforeCommit, lockPath) {
  let original;
  let mode;
  try {
    original = fs.readFileSync(file);
    mode = fs.statSync(file).mode & 0o7777;
  } catch (error) {
    throw new MemoryUpgradeError('memory_unreadable', `memory_unreadable:path=${file}:error=${error.message}`);
  }
  const status = inspectMemoryUpgrade(original, true);
  if (!status.compatible) {
    throw new MemoryUpgradeError('memory_incompatible', status.refusal_reason ?? serverMessage('memory_upgrade_incompatible'), status);
  }
  if (!status.migration_required) return noopReceipt(file, status);
  if (status.detected_schema_version !== 1) {
    throw new MemoryUpgradeError('migration_unavailable', serverMessage('memory_upgrade_unavailable'), status);
  }
  const digest = sha256Hex(original);
  const backup = backupPath ?? defaultBackupPath(file, digest);
  const receiptFile = receiptPath ?? defaultReceiptPath(file);
  const same = (left, right) => path.normalize(left) === path.normalize(right);
  if (same(backup, file) || same(backup, lockPath) || same(receiptFile, file) || same(receiptFile, lockPath) || same(backup, receiptFile)) {
    throw new MemoryUpgradeError('migration_path_collision', serverMessage('memory_upgrade_path_collision'));
  }
  writeVerifiedBackup(backup, original, digest, mode);
  const migrated = migrateV1ToV2(original);
  const migratedStatus = inspectMemoryUpgrade(migrated, true);
  if (!migratedStatus.compatible || migratedStatus.detected_schema_version !== TARGET_SCHEMA
    || migratedStatus.event_count !== status.event_count) {
    throw new MemoryUpgradeError('migration_validation_failed', serverMessage('memory_upgrade_validation_failed'), migratedStatus);
  }
  const receipt = {
    binary_version: packageVersion(),
    changed: true,
    migration_id: V1_TO_V2_MIGRATION_ID,
    from_schema_version: 1,
    to_schema_version: TARGET_SCHEMA,
    memory_path: file,
    backup_path: backup,
    receipt_path: receiptFile,
    original_sha256: digest,
    migrated_sha256: sha256Hex(migrated),
    event_count: status.event_count ?? 0,
    rollback_supported: true,
    rollback_strategy: 'restore_backup_path',
  };
  const staged = temporaryPath(file, 'migration');
  try {
    writeNewFile(staged, migrated, mode);
  } catch (error) {
    throw new MemoryUpgradeError('migration_stage_failed', `migration_stage_failed:path=${staged}:error=${error.message}`);
  }
  const stagedReceipt = temporaryPath(receiptFile, 'receipt');
  try {
    writeNewFile(stagedReceipt, Buffer.from(prettyJson(receipt)), mode);
  } catch (error) {
    fs.rmSync(staged, { force: true });
    throw new MemoryUpgradeError('receipt_stage_failed', `receipt_stage_failed:path=${stagedReceipt}:error=${error.message}`);
  }
  try {
    beforeCommit(staged);
  } catch (error) {
    fs.rmSync(staged, { force: true });
    fs.rmSync(stagedReceipt, { force: true });
    throw new MemoryUpgradeError('migration_interrupted', `migration_interrupted:error=${error.message}`, status);
  }
  try {
    fs.renameSync(staged, file);
  } catch (error) {
    fs.rmSync(staged, { force: true });
    fs.rmSync(stagedReceipt, { force: true });
    throw new MemoryUpgradeError('migration_commit_failed', `migration_commit_failed:path=${file}:error=${error.message}`);
  }
  try {
    syncParent(file);
  } catch (error) {
    fs.rmSync(stagedReceipt, { force: true });
    throw new MemoryUpgradeError('migration_sync_failed', `migration_sync_failed:path=${file}:error=${error.message}`);
  }
  try {
    fs.renameSync(stagedReceipt, receiptFile);
  } catch (error) {
    fs.rmSync(stagedReceipt, { force: true });
    throw new MemoryUpgradeError('receipt_write_failed', `receipt_write_failed:error=${error.message}`);
  }
  try {
    syncParent(receiptFile);
  } catch (error) {
    throw new MemoryUpgradeError('receipt_sync_failed', `receipt_sync_failed:path=${receiptFile}:error=${error.message}`);
  }
  return receipt;
}

/** Parse `--name value` / `--name=value` flags the way clap reads them. */
function parseFlags(argv, names) {
  const values = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const at = arg.indexOf('=');
    const name = at < 0 ? arg : arg.slice(0, at);
    if (!names.includes(name)) return { unsupported: arg };
    values[name] = at < 0 ? argv[++index] : arg.slice(at + 1);
    if (values[name] === undefined) return { unsupported: arg };
  }
  if (values['--format'] !== undefined && values['--format'] !== 'json') return { unsupported: `--format ${values['--format']}` };
  return { values };
}

/**
 * The `formal-ai memory upgrade-status|migrate` CLI (rust/src/cli_memory.rs):
 * pretty JSON on stdout, and on refusal the Rust `main`'s `Error: "<reason>"`
 * on stderr with exit code 1. Unknown flags exit 2, like clap.
 * @param {Array<string>} argv the words after `memory`
 * @returns {Promise<{code: number, stdout: string, stderr: string}>}
 */
export async function runMemoryCli(argv, env = process.env) {
  const [action, ...rest] = argv;
  const flags = action === 'migrate' ? ['--path', '--backup', '--receipt', '--format'] : ['--path', '--format'];
  if (action !== 'upgrade-status' && action !== 'migrate') {
    return { code: 2, stdout: '', stderr: `${serverMessage('memory_cli_unsupported', { argument: action ?? '' })}\n` };
  }
  const parsed = parseFlags(rest, flags);
  if (parsed.unsupported) return { code: 2, stdout: '', stderr: `${serverMessage('memory_cli_unsupported', { argument: parsed.unsupported })}\n` };
  const file = parsed.values['--path'] ?? sharedMemoryPath(env);
  const refused = (key) => `Error: ${JSON.stringify(serverMessage(key))}\n`;
  if (action === 'upgrade-status') {
    const status = preflightMemoryUpgrade(file);
    return { code: status.compatible ? 0 : 1, stdout: `${prettyJson(status)}\n`, stderr: status.compatible ? '' : refused('memory_upgrade_preflight_refused') };
  }
  try {
    const receipt = await migrateMemory(file, parsed.values['--backup'] ?? null, parsed.values['--receipt'] ?? null);
    return { code: 0, stdout: `${prettyJson(receipt)}\n`, stderr: '' };
  } catch (error) {
    if (!(error instanceof MemoryUpgradeError)) throw error;
    const body = { error: { code: error.code, message: error.message }, status: error.status };
    return { code: 1, stdout: `${prettyJson(body)}\n`, stderr: refused('memory_upgrade_migration_refused') };
  }
}
