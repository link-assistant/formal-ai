// One lazy descriptor and callable implementation for every learning report.
import {readText} from './host.mjs';
import {agenticMessage} from './messages.mjs';
import {matches,renderDocument as renderReport,renderDocumentFrom as renderReportFrom,finalAnswer as reportFinalAnswer} from './learning_report.mjs';

/**
 * Mirrors `struct LearningReport` and its descriptor-bound methods in
 * rust/src/agentic_coding/learning_report.rs. The native struct owns matches,
 * render_document, render_document_from and final_answer; the JavaScript
 * object binds those same operations to one lazy report descriptor.
 */
export function createLearningReportApi(descriptor,matchName='isTask') {
  const report={
    head:descriptor.head,
    issue:descriptor.issue,
    promotion_gate:descriptor.promotion_gate,
    path:descriptor.path,
    get task(){return agenticMessage(descriptor.taskMessage);},
    get memory(){return readText(descriptor.memoryPath);},
    get subject(){return agenticMessage(descriptor.subjectMessage);},
  };
  if(Object.hasOwn(descriptor,'taskValue'))Object.defineProperty(report,'task',{value:descriptor.taskValue,writable:true,enumerable:true,configurable:true});
  const REPORT=Object.freeze(report);
  const api={
    REPORT,
    task:function task(){return REPORT.task;},
    isTask:function isTask(prompt){return matches(REPORT,prompt);},
    renderDocument:function renderDocument(){return renderReport(REPORT);},
    renderDocumentFrom:function renderDocumentFrom(memoryDocument){return renderReportFrom(REPORT,memoryDocument);},
    finalAnswer:function finalAnswer(document){return reportFinalAnswer(REPORT,document);},
  };
  Object.defineProperty(api.isTask,'name',{value:matchName,configurable:true});
  return api;
}
