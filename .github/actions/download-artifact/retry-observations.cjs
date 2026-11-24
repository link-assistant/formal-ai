// Scope annotations to the actual bounded download attempt. Keep the action
// exit status: only a later successful attempt permits extraction.
const name = (process.env.INPUT_NAME || '').trim();
const attempt = process.env.ARTIFACT_RETRY_ATTEMPT;
const escape = value => value.replaceAll('%', '%25').replaceAll('\r', '%0D').replaceAll('\n', '%0A');
const reason = 'Unable to download artifact(s): Artifact not found for name: ' + escape(name);
const help = escape('\n        Please ensure that your artifact is not expired and the artifact was uploaded using a compatible version of toolkit/upload-artifact.\n        For more information, visit the GitHub Artifacts FAQ: https://github.com/actions/toolkit/blob/main/packages/artifact/docs/faq.md');
const write = process.stdout.write;
process.stdout.write = function (chunk, ...args) {
  const text = Buffer.isBuffer(chunk) ? chunk.toString('utf8') : chunk;
  if (typeof text === 'string') {
    const line = text.replace(/\r?\n$/u, '');
    const terminated = line !== text;
    if (terminated && name && (attempt === '1' || attempt === '2') &&
        (line === '::error::' + reason || line === '::error::' + reason + help)) {
      chunk = text.replace('::error::', '::notice::Artifact lookup failed; bounded retry remains: ');
    } else if (terminated && /^::warning::Could not find the following artifact IDs: [^\r\n]+$/u.test(line)) {
      // The pinned action otherwise downloads a subset and exits successfully.
      process.exitCode = 1;
      chunk = text.replace('::warning::', '::error::Incomplete requested artifact selection: ');
    }
  }
  return Reflect.apply(write, this, [chunk, ...args]);
};
