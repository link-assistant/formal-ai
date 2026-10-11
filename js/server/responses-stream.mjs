// The Responses SSE event sequence (rust/src/responses_stream.rs
// `responses_sse_response`). Every frame is a `json!` value, so its keys come
// out sorted; the response snapshot is `serde_json::to_value` of the
// `ResponseObject`, sorted the same way.

import { sortedKeys, toCompactJson } from './json.mjs';
import { sseResponse } from './response.mjs';

function pushEvent(state, event, data) {
  state.body += `event: ${event}\ndata: ${toCompactJson(sortedKeys(data))}\n\n`;
}

function nextSequence(state) {
  const current = state.sequence;
  state.sequence += 1;
  return current;
}

function responseSnapshot(response, status, includeOutput) {
  return { ...response, status, output: includeOutput ? response.output : [] };
}

function itemStarted(item) {
  if (item.type === 'message') return { id: item.id, type: item.type, role: item.role, content: [] };
  if (item.type === 'reasoning') return { id: item.id, type: item.type, summary: [] };
  return item;
}

function frame(state, kind, fields) {
  pushEvent(state, kind, { type: kind, sequence_number: nextSequence(state), ...fields });
}

function messageEvents(state, outputIndex, message) {
  message.content.forEach((content, contentIndex) => {
    const at = { item_id: message.id, output_index: outputIndex, content_index: contentIndex };
    frame(state, 'response.content_part.added', { ...at, part: { type: content.type, text: '' } });
    frame(state, 'response.output_text.delta', { ...at, delta: content.text });
    frame(state, 'response.output_text.done', { ...at, text: content.text });
    frame(state, 'response.content_part.done', { ...at, part: content });
  });
}

function functionCallEvents(state, outputIndex, call) {
  const at = { item_id: call.id, output_index: outputIndex };
  frame(state, 'response.function_call_arguments.delta', { ...at, delta: call.arguments });
  frame(state, 'response.function_call_arguments.done', { ...at, arguments: call.arguments });
}

function customToolCallEvents(state, outputIndex, call) {
  const at = { item_id: call.id, output_index: outputIndex };
  frame(state, 'response.custom_tool_call_input.delta', { ...at, delta: call.input });
  frame(state, 'response.custom_tool_call_input.done', { ...at, input: call.input });
}

function reasoningEvents(state, outputIndex, reasoning) {
  reasoning.summary.forEach((summary, summaryIndex) => {
    const at = { item_id: reasoning.id, output_index: outputIndex, summary_index: summaryIndex };
    frame(state, 'response.reasoning_summary_part.added', { ...at, part: { type: summary.type, text: '' } });
    frame(state, 'response.reasoning_summary_text.delta', { ...at, delta: summary.text });
    frame(state, 'response.reasoning_summary_text.done', { ...at, text: summary.text });
    frame(state, 'response.reasoning_summary_part.done', { ...at, part: summary });
  });
}

function outputItemEvents(state, outputIndex, item) {
  frame(state, 'response.output_item.added', { output_index: outputIndex, item: itemStarted(item) });
  if (item.type === 'message') {
    messageEvents(state, outputIndex, item);
  } else if (item.type === 'function_call') {
    functionCallEvents(state, outputIndex, item);
  } else if (item.type === 'custom_tool_call') {
    customToolCallEvents(state, outputIndex, item);
  } else if (item.type === 'web_search_call') {
    for (const phase of ['in_progress', 'searching', 'completed']) {
      frame(state, `response.web_search_call.${phase}`, { item_id: item.id, output_index: outputIndex });
    }
  } else if (item.type === 'reasoning') {
    reasoningEvents(state, outputIndex, item);
  }
  frame(state, 'response.output_item.done', { output_index: outputIndex, item });
}

/** `responses_sse_response`: reasoning items stream before the rest. */
export function responsesSse(response) {
  const state = { body: '', sequence: 0 };
  frame(state, 'response.created', { response: responseSnapshot(response, 'in_progress', false) });
  frame(state, 'response.in_progress', { response: responseSnapshot(response, 'in_progress', false) });
  response.output.forEach((item, index) => {
    if (item.type === 'reasoning') outputItemEvents(state, index, item);
  });
  response.output.forEach((item, index) => {
    if (item.type !== 'reasoning') outputItemEvents(state, index, item);
  });
  frame(state, 'response.completed', { response: responseSnapshot(response, 'completed', true) });
  return sseResponse(state.body);
}
