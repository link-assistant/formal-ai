import test from 'node:test';
import assert from 'node:assert/strict';
import { completedBatchTrace, batchTraceLine } from '../../../scripts/lib/coverage-batch-trace.mjs';
const context = {source:'observed-head',runId:'7',runAttempt:'2'};
const batch = {index:4,target:'unit',names:['odd\nname'],argv:['--exact','odd\nname'],
  executable:'/observed/executable',selected:1,completed:1,elapsedSeconds:2.75,
  processId:17,exitCode:0,signal:null,summary:{passed:1,failed:0,ignored:0},
  problems:[],succeeded:true};
test('completed singleton preserves exact process and escaped names', () => {
  const line = batchTraceLine(context,batch);
  assert.equal(line.split('\n').length,2);
  const trace = JSON.parse(line.slice('COVERAGE_BATCH_TRACE '.length));
  assert.deepEqual(trace.batch,batch);
  assert.deepEqual(trace.context,context);
  assert.equal(trace.timingScope,'single-completed-case-process');
  assert.equal(trace.executionSourceAuthentication,'Unknown');
});
test('group duration remains indivisible', () => {
  const grouped = {...batch,names:['a','b'],selected:2,completed:2,summary:{passed:2,failed:0,ignored:0}};
  assert.equal(completedBatchTrace(context,grouped).timingScope,'batch-process-only');
});
test('failed missing ignored and incomplete completions never grant singleton timing', () => {
  for (const change of [{succeeded:false},{exitCode:1},{signal:'SIGTERM'},
    {error:'launch failed'},{completed:0},{summary:{passed:0,failed:0,ignored:1}},
    {problems:['missing completion']}]) {
    assert.equal(completedBatchTrace(context,{...batch,...change}).timingScope,'batch-process-only');
  }
});
test('malformed names and process durations refuse', () => {
  for (const change of [{names:[]},{names:['a','a'],selected:2},
    {elapsedSeconds:Infinity},{elapsedSeconds:-1},{argv:[null]}]) {
    assert.throws(() => completedBatchTrace(context,{...batch,...change}));
  }
});

test('sparse names and sparse argument arrays are refused', () => {
  assert.throws(() => completedBatchTrace(context,{...batch,names:Array(1)}));
  assert.throws(() => completedBatchTrace(context,{...batch,argv:Array(1)}));
});
