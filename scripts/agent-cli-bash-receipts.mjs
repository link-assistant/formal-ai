// Checked repair for the pinned AgentCLI producer; consumer adapters retain output unchanged.
import {readFileSync,writeFileSync,renameSync,existsSync,realpathSync,statSync} from 'node:fs';
import {join,resolve,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const sha=(bytes)=>createHash('sha256').update(bytes).digest('hex');
const preimages={
 'src/tool/bash.ts':'f8e74577db978d9fda233b6e2e80d6cd0096d6727ade041dd32c248dd859f07a',
 'src/session/prompt.ts':'6d78578a4d31ebfcec8a68af2ef9a30c18f58b2c74f9ec4482ff172d736e312a',
 'src/session/message-v2.ts':'1d2f7e37dedefaeee353b34a7f5d36a9be699c44455fb967a05aec702a01e638',
};
const patchedBashSha='ad35c659dab7da56ffd15f2fe3eada729922a8186340d1c22eb4c53cf9cfe55e';
const helper=readFileSync(new URL('./lib/agent-cli-bash-receipt.mjs',import.meta.url));
function patchedBash(original) {
 const boundary='    const proc = spawn(params.command';
 if(original.indexOf(boundary)!==original.lastIndexOf(boundary)||!original.includes(boundary))throw new Error('ambiguous Bash producer boundary');
 return original.slice(0,original.indexOf(boundary))
  .replace("import { spawn } from 'child_process';","import { executeShellReceipt, encodeExecutionReceipt } from './bash-execution-receipt.mjs';")
  .replace('const SIGKILL_TIMEOUT_MS = 200;\n','')+`    const receipt = await executeShellReceipt({
      command: params.command, cwd: Instance.directory, timeout, abort: ctx.abort,
      maxOutputLength: MAX_OUTPUT_LENGTH,
      onOutput(output: string) { ctx.metadata({ metadata: { output, description: params.description } }); },
    });
    const output = encodeExecutionReceipt(receipt);
    return {
      title: params.command,
      metadata: { output, exit: receipt.exit_code, description: params.description },
      output,
    };
  },
});
`;
}
export function installBashReceipts(packageRoot) {
 const entry=realpathSync(packageRoot);
 const root=statSync(entry).isFile()?dirname(dirname(entry)):entry;
 const info=JSON.parse(readFileSync(join(root,'package.json'),'utf8'));
 if(info.name!=='@link-assistant/agent'||info.version!=='0.26.0')throw new Error('AgentCLI package/version differs from checked 0.26.0 contract');
 for(const path of ['src/session/prompt.ts','src/session/message-v2.ts']) {
  if(sha(readFileSync(join(root,path)))!==preimages[path])throw new Error('AgentCLI adapter preimage differs: '+path);
 }
 const target=join(root,'src/tool/bash.ts'),helperTarget=join(root,'src/tool/bash-execution-receipt.mjs');
 const original=readFileSync(target,'utf8');
 // A checked prior install is idempotent; no unrecognized source is rewritten.
 if(original.includes("from './bash-execution-receipt.mjs'")) {
  const installedHelper=readFileSync(helperTarget);
  if(sha(original)!==patchedBashSha||!installedHelper.equals(helper))throw new Error('installed Bash receipt differs from checked repair');
  return {version:info.version,already_installed:true,bash_sha256:sha(original),helper_sha256:sha(helper)};
 }
 if(sha(original)!==preimages['src/tool/bash.ts'])throw new Error('AgentCLI Bash preimage differs');
 if(existsSync(helperTarget)&&!readFileSync(helperTarget).equals(helper))throw new Error('unknown existing Bash receipt helper');
 const patched=patchedBash(original);
 if(sha(patched)!==patchedBashSha)throw new Error('derived Bash repair hash differs');
 // Helper arrives first; the live consumer is replaced only after both bytes are ready.
 const temporary=target+'.receipt-'+process.pid+'.tmp';
 writeFileSync(helperTarget,helper);writeFileSync(temporary,patched);renameSync(temporary,target);
 return {version:info.version,already_installed:false,bash_sha256:sha(patched),helper_sha256:sha(helper),original_sha256:preimages['src/tool/bash.ts']};
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
 if(process.argv.length!==3)throw new Error('usage: node scripts/agent-cli-bash-receipts.mjs <installed-agent-package-root>');
 console.log(JSON.stringify(installBashReceipts(process.argv[2])));
}
