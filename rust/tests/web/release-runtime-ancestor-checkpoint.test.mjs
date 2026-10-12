import test from 'node:test';
import assert from 'node:assert/strict';
import { observeActiveAncestor, validateQueueSnapshot } from '../../../scripts/release-runtime-fixture.mjs';
const context = {
  repository: 'link-assistant/formal-ai',
  run: '123'
};
const name = 'formal-ai-release-fixture-123-1';
const member = {
  run_id: 123,
  status: 'in_progress'
};
const state = {
  status: 200,
  body: {
    group_name: name,
    group_members: [member]
  }
};
const job = {
  id: 456,
  run_id: 123,
  name: 'Active reusable fixture / Fixture lease holder active',
  status: 'in_progress'
};
async function observe({
  jobs = [job],
  ancestor = state
} = {}) {
  const calls = [],
    snapshots = [];
  const get = async route => {
    calls.push(route);
    return route.includes('/jobs?') ? {
      status: 200,
      body: {
        jobs
      }
    } : ancestor;
  };
  let result, error;
  try {
    result = await observeActiveAncestor({
      get,
      context,
      name,
      label: 'Fixture lease holder active',
      snapshots
    });
  } catch (e) {
    error = e;
  }
  return {
    calls,
    snapshots,
    result,
    error
  };
}
test('checkpoint queries active descendant while independent contender remains absent', async () => {
  const x = await observe();
  assert.equal(x.result.observation, 'VerifiedPrivateFixtureAncestorCheckpoint');
  assert.equal(x.calls[1], '/repos/link-assistant/formal-ai/actions/concurrency_groups/' + name + '?ahead_of_job=456');
  assert.throws(() => validateQueueSnapshot(state, {
    group: name,
    run: '123'
  }));
});
for (const [label, jobs] of [['missing', []], ['completed', [{
  ...job,
  status: 'completed'
}]], ['foreign run', [{
  ...job,
  run_id: 124
}]], ['ambiguous', [job, {
  ...job,
  id: 457
}]], ['invalid ID', [{
  ...job,
  id: NaN
}]]]) test(label + ' descendant refuses', async () => {
  const x = await observe({
    jobs
  });
  assert.ok(x.error);
  assert.equal(x.calls.length, 1);
});
for (const [label, ancestor] of [['422', {
  status: 422,
  body: {
    message: 'not in group'
  }
}], ['empty', {
  status: 200,
  body: {
    group_name: name,
    group_members: []
  }
}], ['foreign group', {
  status: 200,
  body: {
    ...state.body,
    group_name: 'other'
  }
}], ['foreign run', {
  status: 200,
  body: {
    ...state.body,
    group_members: [{
      ...member,
      run_id: 124
    }]
  }
}], ['pending only', {
  status: 200,
  body: {
    ...state.body,
    group_members: [{
      ...member,
      status: 'pending'
    }]
  }
}]]) test(label + ' ancestor stays refused', async () => {
  const x = await observe({
    ancestor
  });
  assert.ok(x.error);
  assert.equal(x.calls.length, 2);
});
test('separate queue obligation still requires actual pending contender', () => {
  const queued = structuredClone(state);
  queued.body.group_members.push({
    run_id: 123,
    status: 'pending',
    job_name: 'Fixture private queued contender',
    job_id: 789
  });
  assert.equal(validateQueueSnapshot(queued, {
    group: name,
    run: '123'
  }), true);
});
test('live 422 retains independent diagnostic operation before refusing authority', async () => {
  const snapshots = [],
    diagnosed = [];
  const get = async route => route.includes('/jobs?') ? {
    status: 200,
    body: {
      jobs: [job]
    }
  } : {
    status: 422,
    body: {
      message: 'not in group'
    }
  };
  await assert.rejects(observeActiveAncestor({
    get,
    context,
    name,
    label: 'Fixture lease holder active',
    snapshots,
    diagnose: async child => diagnosed.push(child.id)
  }));
  assert.deepEqual(diagnosed, [456]);
  assert.equal(snapshots[1].status, 422);
});
test('preceding active same-run member cannot substitute for foreign matched target', async () => {
  const ancestor = {
    status: 200,
    body: {
      group_name: name,
      group_members: [member, {
        run_id: 124,
        status: 'pending'
      }]
    }
  };
  const x = await observe({
    ancestor
  });
  assert.ok(x.error);
});
test('preceding active same-run member cannot substitute for pending matched target', async () => {
  const ancestor = {
    status: 200,
    body: {
      group_name: name,
      group_members: [member, {
        run_id: 123,
        status: 'pending'
      }]
    }
  };
  const x = await observe({
    ancestor
  });
  assert.ok(x.error);
});
