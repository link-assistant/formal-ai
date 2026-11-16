// Issue #1162 / #1156: does each registered language's "again" word make a
// later mention of an already bound output a further line, while a plain
// repeated mention stays one output obligation?
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { boundOutputLiterals } from '../../js/agentic/crate/intent_formalization_obligations.mjs';

await installNodeHost(new WorkerHost());

const requests = {
  en: ['Write a Kotlin program that prints `Hi` and then prints `Hi` again.', 'Write a Kotlin program that prints `Hi`. The program prints `Hi` exactly.'],
  ru: ['Напиши программу на Kotlin, которая выводит `Hi`, а затем выводит `Hi` снова.', 'Напиши программу на Kotlin, которая выводит `Hi`. Программа выводит `Hi` точно.'],
  hi: ['एक Kotlin प्रोग्राम लिखें जो `Hi` प्रिंट करे और फिर `Hi` दोबारा प्रिंट करे।', 'एक Kotlin प्रोग्राम लिखें जो `Hi` प्रिंट करे। प्रोग्राम `Hi` प्रिंट करता है।'],
  zh: ['编写一个 Kotlin 程序，打印 `Hi`，然后打印 `Hi` 再一次。', '编写一个 Kotlin 程序，打印 `Hi`。程序打印 `Hi`。'],
  zh2: ['编写一个 Kotlin 程序，打印 `Hi`，然后再次打印 `Hi`。', ''],
  es: ['Escribe un programa en Kotlin que imprime `Hi` y luego imprime `Hi` de nuevo.', 'Escribe un programa en Kotlin que imprime `Hi`. El programa imprime `Hi` exactamente.'],
  es2: ['Escribe un programa en Kotlin que imprime `Hi` y luego imprime `Hi` otra vez.', 'Escribe un programa en Kotlin que imprima `Hi`.'],
};
for (const [language, [again, plain]] of Object.entries(requests)) {
  console.log(language, JSON.stringify(boundOutputLiterals(again)), JSON.stringify(boundOutputLiterals(plain)));
}
