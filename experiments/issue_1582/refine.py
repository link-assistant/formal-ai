"""Supplied follow-up repair recipe, with zero autonomous authorship credit."""
from pathlib import Path
def replace(file,old,new):
 p=Path(file);s=p.read_text();assert old in s,(file,old[:80]);p.write_text(s.replace(old,new))

# English how-to promotion runs before feature status. Keep the narrow, seeded
# permission query ahead of that generic promotion without hijacking other how-to.
replace('js/worker/formal_ai_worker_solve.js','  const nlTool = tryNaturalLanguageToolRequest', '''  if (lexiconMentionsRole("internet_access_help", normalized)) {
    const access = tryFeatureCapabilityStatus(prompt, normalized, language, preferences);
    return finalizeInlineHandler(events, steps, toolCalls, access, "tryFeatureCapabilityStatus", formalizationContext);
  }
  const nlTool = tryNaturalLanguageToolRequest''')
replace('rust/src/solver_handler_how.rs','    let task = extract_procedural_how_to_task(normalized)?;', '''    // Permission controls belong to capability guidance, before general how-to.
    if crate::seed::lexicon().mentions_role("internet_access_help", normalized) {
        return None;
    }
    let task = extract_procedural_how_to_task(normalized)?;''')

replace('rust/tests/web/issue-1582-defaults.test.mjs',"import {WorkerHost}","import {parseLinks,writeLink} from '../../../js/logging/lino-codec.mjs';\nimport {spawnSync} from 'node:child_process';\nimport {WorkerHost}")
replace('rust/tests/web/issue-1582-defaults.test.mjs',"  const writes=[];",'''  const child=spawnSync(process.execPath,['--input-type=module','-e',
    `import {createServeLogger} from './js/server/structured-log.mjs'; const logger=createServeLogger({env:{HOME:${JSON.stringify(directory)}}}); logger.log('debug','wrapper.started'); logger.close();`],{encoding:'utf8'});
  assert.equal(child.status,0);assert.equal(child.stderr,'');assert.equal(child.stdout,'');
  const writes=[];''')
replace('rust/tests/web/issue-1582-defaults.test.mjs',"decodeTyped(text.trimEnd()).metadata.secret", "decodeTyped(writeLink(parseLinks(text)[0])).metadata.secret")

# Read persisted diagnostics in CLI fixtures, so tests exercise both the quiet
# terminal contract and the actual file evidence instead of requiring a tee.
replace('rust/tests/web/support/structured-log-fixture.mjs',"return stderr.split('\\n').slice(0,-1).filter(Boolean).map(decodeTyped);",'''const explicit=extra.indexOf('--log-dir');
      const logDirectory=explicit<0?path.join(home,'.formal-ai/logs'):extra[explicit+1];
      const file=path.join(logDirectory,'serve-'+child.pid+'.lino');
      const text=fs.existsSync(file)?fs.readFileSync(file,'utf8'):stderr;
      return text.split('\\n').slice(0,-1).filter(Boolean).map(decodeTyped);''')
replace('rust/tests/web/server-structured-log-cli.test.mjs',"CLI defaults emit typed debug tee and preserve", "CLI defaults persist typed debug records quietly and preserve")
replace('rust/tests/web/server-structured-log-cli.test.mjs',"assert.equal(text,run.stderr);", "assert.equal(run.stderr,'');assert(text.length>0);")
replace('rust/tests/web/server-structured-log-cli.test.mjs',"assert.equal(persisted,run.stderr);", "assert.equal(run.stderr,'');assert(persisted.length>0);")
