// Supplied finite probe: real language detection, without a forced response language.
import {WorkerHost} from '../../js/server/worker-host.mjs';
const questions = [
  ['en', 'How do I enable internet access?'],
  ['ru', 'Как включить доступ к интернету?'],
  ['hi', 'इंटरनेट की अनुमति कैसे दें?'],
  ['zh', '如何启用互联网访问？'],
  ['es', '¿Cómo habilitar el acceso a internet?'],
  ['ru', 'Что мне сделать, чтобы дать тебе доступ в интернет?'],
  ['en', 'Allow internet access'],
];
const host = new WorkerHost({WebAssembly:{instantiate(){throw Error('LocalWasmExecutionForbidden');},compile(){throw Error('LocalWasmExecutionForbidden');}}});
for (const [language, prompt] of questions) {
  const result = await host.solve(prompt);
  console.log(JSON.stringify({language,prompt,intent:result.intent,content:result.content,evidence:result.evidence}));
}
