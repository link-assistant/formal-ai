// Probe: which report phrasings route to the report flow (issue #1632 R6).
import { WorkerHost } from '../js/server/worker-host.mjs';
import { installNodeHost } from '../js/agentic/node-host.mjs';
import { isReportIntent, planReportFlow } from '../js/agentic/report_issue.mjs';
await installNodeHost(new WorkerHost());
for (const p of ['Отправь отчёт об ошибке', 'Отправь отчет об ошибке', 'Сообщи об ошибке', 'Send a bug report', 'Submit an error report', 'Отправить отчёт об ошибке']) {
  const plan = planReportFlow([{ role: 'user', content: p }], ['AskUserQuestion', 'Bash']);
  console.log(p, isReportIntent(p), JSON.stringify(plan)?.slice(0, 300));
}
for (const p of ['Отправь письмо маме', 'Отправь отчёт начальнику', 'Создай файл', 'Отправь отчёт', 'Отправь, пожалуйста, отчёт об ошибке разработчикам']) console.log('NEG?', p, isReportIntent(p));
