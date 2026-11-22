import { spawnSync } from 'node:child_process';
import { readNetworkAllowed } from './read-access.mjs';

const BODY_BYTE_LIMIT = 1024 * 1024;
export class ComputerUseHttpError extends Error {}

function parseHttpUrl(value) {
  try {
    const url = new URL(value);
    if (['http:', 'https:'].includes(url.protocol) && !url.username && !url.password && !url.href.includes('#')) return url;
  } catch {
    return null;
  }
  return null;
}

export function computerUseHttpTarget(value, env = process.env) {
  if (!readNetworkAllowed(env)) throw new ComputerUseHttpError('computer_use_network_denied');
  const origin = parseHttpUrl(env.FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN);
  if (!origin || origin.pathname !== '/' || origin.href.includes('?')) throw new ComputerUseHttpError('computer_use_http_origin_required');
  const target = parseHttpUrl(value);
  if (!target || target.origin !== origin.origin) throw new ComputerUseHttpError('computer_use_http_origin_mismatch');
  return target;
}

export function computerUseHttpRequest(target, body) {
  if (body !== null && Buffer.byteLength(body) > BODY_BYTE_LIMIT) {
    throw new ComputerUseHttpError('computer_use_http_body_too_large');
  }
  const options = ['-q', '--silent', '--show-error', '--globoff', '--proto', '=http,https',
    '--max-time', '15', '--max-filesize', String(BODY_BYTE_LIMIT),
    '--write-out', '%{stderr}\n%{http_code}\n%{content_type}'];
  if (body !== null) options.push('--data-binary', '@-');
  options.push('--', target.href);
  const result = spawnSync('curl', options, { input: body ?? undefined, maxBuffer: BODY_BYTE_LIMIT, timeout: 16_000 });
  if (result.error || result.status !== 0) throw new ComputerUseHttpError('computer_use_http_transport_failed');
  const metadata = /^\n(\d{3})\n([^\r\n]*)$/u.exec(result.stderr.toString('utf8'));
  if (!metadata) throw new ComputerUseHttpError('computer_use_http_metadata_missing');
  const status = Number(metadata[1]);
  if (status < 200 || status >= 300) throw new ComputerUseHttpError(`computer_use_http_status:${status}`);
  try {
    const text = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true }).decode(result.stdout);
    return { status, contentType: metadata[2], text };
  } catch {
    throw new ComputerUseHttpError('computer_use_http_body_not_utf8');
  }
}
