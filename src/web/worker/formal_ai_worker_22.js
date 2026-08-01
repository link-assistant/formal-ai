// Worker module 22. Issue #708 browser mirror of `src/memory_program.rs` and
// `src/memory_program/execution.rs`. Natural-language surfaces, primitive
// permissions, and ordered programs come exclusively from memory-programs.lino.

const MEMORY_PROGRAM_DEFAULT_LIMITS = {
  maxMatches: 4 * 32,
  maxIterations: 4,
};

let cachedMemoryProgramCatalog = null;
let cachedMemoryProgramCompilation = null;

function memoryProgramCatalog() {
  if (cachedMemoryProgramCatalog) return cachedMemoryProgramCatalog;
  const raw = SEED_RAW["seed/memory-programs.lino"] || "";
  if (!raw || !self.FormalAiSeed) return null;
  const root = self.FormalAiSeed.parse(raw);
  const catalog = { primitives: {}, cues: [], families: [] };
  for (const node of root.children || []) {
    if (node.name === "primitive") {
      const permission = (node.children || []).find(
        (child) => child.name === "permission",
      );
      if (node.value && permission) {
        catalog.primitives[node.value] = permission.value;
      }
    } else if (node.name === "cue" && node.value) {
      catalog.cues.push(memoryProgramNormalizeSurface(node.value));
    } else if (node.name === "family") {
      const family = { id: node.value, steps: [], templates: [] };
      for (const child of node.children || []) {
        if (child.name.startsWith("step_")) family.steps.push(child.value);
        if (child.name.startsWith("template_")) {
          family.templates.push(memoryProgramNormalizeSurface(child.value));
        }
      }
      catalog.families.push(family);
    }
  }
  cachedMemoryProgramCatalog = catalog;
  return catalog;
}

function memoryProgramNormalizeSurface(value) {
  return String(value || "")
    .trim()
    .replace(/[.!?;।。！？]+$/u, "")
    .replace(/\s+/gu, " ");
}

function memoryProgramMatchTemplate(template, request) {
  const slotPattern = /\{([^}]+)\}/g;
  const names = [];
  const literals = [];
  let cursor = 0;
  let match;
  while ((match = slotPattern.exec(template)) !== null) {
    literals.push(template.slice(cursor, match.index));
    names.push(match[1]);
    cursor = match.index + match[0].length;
  }
  literals.push(template.slice(cursor));
  if (names.length === 0) {
    return template.toLocaleLowerCase() === request.toLocaleLowerCase() ? {} : null;
  }
  const escape = (text) => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  let source = "^";
  for (let index = 0; index < names.length; index += 1) {
    source += escape(literals[index]) + "(.+?)";
  }
  source += escape(literals[literals.length - 1]) + "$";
  const captures = new RegExp(source, "iu").exec(request);
  if (!captures) return null;
  const bindings = {};
  for (let index = 0; index < names.length; index += 1) {
    const value = captures[index + 1].trim();
    const previous = bindings[names[index]];
    if (
      previous !== undefined &&
      memoryProgramNormalizeBinding(previous) !== memoryProgramNormalizeBinding(value)
    ) {
      return null;
    }
    bindings[names[index]] = value;
  }
  return bindings;
}

function memoryProgramNormalizeBinding(value) {
  return memoryProgramNormalizeSurface(value).toLocaleLowerCase();
}

function memoryProgramCanonical(family, limits, bindings, steps) {
  let result =
    `family=${family}\nmax_matches=${limits.maxMatches}\n` +
    `max_iterations=${limits.maxIterations}\n`;
  for (const name of Object.keys(bindings).sort()) {
    result += `binding:${name}=${memoryProgramNormalizeBinding(bindings[name])}\n`;
  }
  for (const step of steps) {
    result += `step:${step.primitive}:${step.permission}`;
    for (const name of Object.keys(step.arguments).sort()) {
      result += `:${name}=${memoryProgramNormalizeBinding(step.arguments[name])}`;
    }
    result += "\n";
  }
  return result;
}

function compileMemoryProgramForWorker(request, limits = MEMORY_PROGRAM_DEFAULT_LIMITS) {
  const catalog = memoryProgramCatalog();
  if (!catalog) return { status: "not_memory_program" };
  const surface = memoryProgramNormalizeSurface(request);
  for (const family of catalog.families) {
    for (const template of family.templates) {
      const bindings = memoryProgramMatchTemplate(template, surface);
      if (bindings === null) continue;
      const steps = [];
      for (const specification of family.steps) {
        const fields = specification.split(/\s+/u);
        const primitive = fields.shift() || "";
        const permission = catalog.primitives[primitive];
        if (!permission) {
          return {
            status: "gap",
            gap: `program_gap:unseeded_memory_primitive:${primitive}`,
          };
        }
        const arguments_ = {};
        for (const field of fields) {
          const separator = field.indexOf("=");
          if (separator < 0) continue;
          const name = field.slice(0, separator);
          const value = field.slice(separator + 1);
          arguments_[name] = value.startsWith("$")
            ? bindings[value.slice(1)]
            : value;
        }
        steps.push({ primitive, permission, arguments: arguments_ });
      }
      const canonical = memoryProgramCanonical(family.id, limits, bindings, steps);
      return {
        status: "compiled",
        program: {
          id: stableBehaviorRuleId("memory_program", canonical),
          family: family.id,
          limits,
          bindings,
          steps,
          canonical,
        },
      };
    }
  }
  const normalized = surface.toLocaleLowerCase();
  if (catalog.cues.some((cue) => normalized.includes(cue))) {
    return {
      status: "gap",
      gap: "program_gap:no_complete_seeded_family",
    };
  }
  return { status: "not_memory_program" };
}

function compileMemoryProgramOnce(prompt) {
  if (cachedMemoryProgramCompilation?.prompt === prompt) {
    return cachedMemoryProgramCompilation.result;
  }
  const result = compileMemoryProgramForWorker(prompt);
  cachedMemoryProgramCompilation = { prompt, result };
  return result;
}

function memoryProgramLinoValue(value) {
  return JSON.stringify(String(value ?? ""));
}

function memoryProgramLinksNotation(program) {
  const lines = [
    "memory_program",
    `  id ${memoryProgramLinoValue(program.id)}`,
    `  family ${memoryProgramLinoValue(program.family)}`,
    `  max_matches ${program.limits.maxMatches}`,
    `  max_iterations ${program.limits.maxIterations}`,
  ];
  for (const name of Object.keys(program.bindings).sort()) {
    lines.push(`  binding ${memoryProgramLinoValue(name)}`);
    lines.push(`    value ${memoryProgramLinoValue(program.bindings[name])}`);
  }
  program.steps.forEach((step, index) => {
    lines.push(`  step ${index + 1}`);
    lines.push(`    primitive ${memoryProgramLinoValue(step.primitive)}`);
    lines.push(`    permission ${memoryProgramLinoValue(step.permission)}`);
    for (const name of Object.keys(step.arguments).sort()) {
      lines.push(`    ${name} ${memoryProgramLinoValue(step.arguments[name])}`);
    }
  });
  const update = program.steps.find((step) => step.primitive === "update");
  if (update?.arguments.old !== undefined && update.arguments.new !== undefined) {
    lines.push("  replace");
    lines.push(`    old ${memoryProgramLinoValue(update.arguments.old)}`);
    lines.push(`    new ${memoryProgramLinoValue(update.arguments.new)}`);
  }
  const effect = program.steps.find((step) =>
    ["create", "update", "delete_with_retraction"].includes(step.primitive),
  );
  if (program.steps.length > 0 && effect) {
    lines.push(`  when ${memoryProgramLinoValue(program.steps[0].primitive)}`);
    lines.push(`    do ${memoryProgramLinoValue(effect.primitive)}`);
  }
  return lines.join("\n");
}

function memoryProgramSourceId(event, index) {
  return String(event?.id ?? index + 1);
}

function memoryProgramEventText(event) {
  return String(event?.content ?? event?.outputs ?? event?.inputs ?? "");
}

function memoryProgramContains(text, pattern) {
  return String(text || "").toLocaleLowerCase().includes(
    String(pattern || "").toLocaleLowerCase(),
  );
}

function memoryProgramCurrentWeek(timestamp) {
  const date = new Date(String(timestamp || ""));
  if (Number.isNaN(date.valueOf())) return false;
  const now = new Date();
  const day = (now.getUTCDay() + 6) % 7;
  const start = Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate() - day);
  return date.valueOf() >= start && date.valueOf() < start + 7 * 86400000;
}

function memoryProgramActiveIndices(events) {
  const retracted = new Set(
    events
      .filter((event) => event?.kind === "memory_retraction")
      .map((event) => String(event.inputs || "")),
  );
  return events
    .map((event, index) => ({ event, index }))
    .filter(({ event, index }) =>
      event?.kind !== "memory_retraction" &&
      !retracted.has(memoryProgramSourceId(event, index)),
    )
    .map(({ index }) => index);
}

function memoryProgramEventMatches(event, arguments_) {
  return Object.entries(arguments_).every(([name, value]) => {
    if (name === "contains") return memoryProgramContains(memoryProgramEventText(event), value);
    if (name === "kind") return event?.kind === value;
    if (name === "sent_at") return String(event?.sentAt || "").startsWith(value);
    if (name === "period" && value === "this_week") {
      return memoryProgramCurrentWeek(event?.sentAt);
    }
    if (name === "field") {
      return ["label", "content"].includes(value)
        ? typeof event?.content === "string"
        : typeof event?.[value] === "string";
    }
    return false;
  });
}

function memoryProgramMarkerPresent(event, marker) {
  return [event?.content, event?.inputs, event?.outputs, event?.demoLabel]
    .concat(Array.isArray(event?.evidence) ? event.evidence : [])
    .some((value) => memoryProgramContains(value, marker));
}

function memoryProgramFilter(events, selection, arguments_) {
  const counts = {};
  for (const index of selection) {
    const text = memoryProgramEventText(events[index]);
    counts[text] = (counts[text] || 0) + 1;
  }
  return selection.filter((index) => {
    const event = events[index] || {};
    return Object.entries(arguments_).every(([name, value]) => {
      if (name === "role") return event.role === value;
      if (name === "kind") return event.kind === value;
      if (name === "missing") {
        return !memoryProgramMarkerPresent(event, value.replace(/_/g, ":"));
      }
      if (name === "duplicate") {
        return value === "true" && counts[memoryProgramEventText(event)] > 1;
      }
      if (name === "links") {
        return value === "none" && (!Array.isArray(event.evidence) || event.evidence.length === 0);
      }
      return false;
    });
  });
}

function memoryProgramReplace(text, oldValue, newValue) {
  if (!oldValue) return null;
  const expression = new RegExp(
    String(oldValue).replace(/[.*+?^${}()|[\]\\]/g, "\\$&"),
    "giu",
  );
  const replaced = String(text || "").replace(expression, String(newValue || ""));
  return replaced === text ? null : replaced;
}

function memoryProgramUpdate(events, selection, arguments_) {
  let changed = 0;
  for (const index of selection) {
    const event = events[index];
    if (!event) continue;
    let eventChanged = false;
    if (arguments_.old !== undefined && arguments_.new !== undefined) {
      const replaced = memoryProgramReplace(event.content, arguments_.old, arguments_.new);
      if (replaced !== null) {
        event.content = replaced;
        eventChanged = true;
      }
    }
    if (arguments_.value !== undefined && event.kind !== arguments_.value) {
      event.kind = arguments_.value;
      eventChanged = true;
    }
    if (arguments_.append !== undefined) {
      const parts = String(event.content || "").split(/\s+/u);
      if (!parts.includes(arguments_.append)) {
        event.content = `${event.content || ""} ${arguments_.append}`.trim();
        eventChanged = true;
      }
    }
    if (arguments_.normalize === "whitespace") {
      const normalized = String(event.content || "").trim().replace(/\s+/gu, " ");
      if (normalized !== event.content) {
        event.content = normalized;
        eventChanged = true;
      }
    }
    if (eventChanged) {
      event.writeCount = Math.max(1, Number(event.writeCount) || 1) + 1;
      changed += 1;
    }
  }
  return changed;
}

function memoryProgramAppendDerived(events, kind, identity, target, output, content) {
  const marker = `memory_program_result:${stableBehaviorRuleId("memory_program_result", identity)}`;
  if (events.some((event) => event?.evidence?.includes(marker))) return 0;
  events.push({
    kind,
    role: "system",
    intent: "memory_program",
    inputs: target || undefined,
    outputs: output || undefined,
    content,
    sentAt: new Date().toISOString(),
    evidence: ["memory_program", marker],
    writeCount: 1,
  });
  return 1;
}

function memoryProgramCreate(events, selection, projection, arguments_) {
  const kind = arguments_.kind || "memory_program_result";
  if (projection.aggregate === "count") {
    const counts = {};
    for (const index of selection) {
      const event = events[index] || {};
      const group = projection.group === "topic"
        ? event.intent || "unclassified"
        : projection.group === "contributor" ? event.role || "unknown" : "unknown";
      counts[group] = (counts[group] || 0) + 1;
    }
    const content = Object.keys(counts).sort().map((key) => `${key}=${counts[key]}`).join(", ");
    return memoryProgramAppendDerived(events, kind, `${kind}:${content}`, null, null, content);
  }
  if (projection.copy === "true") {
    return selection.reduce((changed, index) => {
      const event = events[index] || {};
      const target = memoryProgramSourceId(event, index);
      return changed + memoryProgramAppendDerived(
        events,
        "collection_member",
        `collection_member:${arguments_.collection || ""}:${target}`,
        target,
        arguments_.collection,
        String(event.content || memoryProgramEventText(event)),
      );
    }, 0);
  }
  const targets = selection.map((index) => memoryProgramSourceId(events[index], index));
  return targets.reduce(
    (changed, target) => changed + memoryProgramAppendDerived(
      events,
      kind,
      `${kind}:${target}`,
      target,
      null,
      `memory_program_result:${kind}:${target}`,
    ),
    0,
  );
}

function memoryProgramRetract(events, selection, arguments_) {
  const reason = arguments_.reason || "memory_program";
  const existing = new Set(
    events
      .filter((event) => event?.kind === "memory_retraction")
      .map((event) => String(event.inputs || "")),
  );
  let changed = 0;
  for (const index of selection) {
    const target = memoryProgramSourceId(events[index], index);
    if (existing.has(target)) continue;
    events.push({
      kind: "memory_retraction",
      role: "system",
      intent: "memory_program",
      inputs: target,
      outputs: reason,
      content: `memory_retraction:${target}`,
      sentAt: new Date().toISOString(),
      evidence: ["policy:append_only_retraction"],
      writeCount: 1,
    });
    existing.add(target);
    changed += 1;
  }
  return changed;
}

function executeMemoryProgramForWorker(program, sourceEvents, destructiveConfirmed = false) {
  if (
    program.steps.some((step) => step.permission === "destructive") &&
    !destructiveConfirmed
  ) {
    return {
      programId: program.id,
      matched: 0,
      changed: 0,
      iterations: 0,
      halt: "permission_denied",
      required: "destructive",
      events: sourceEvents,
      matchedEventIds: [],
    };
  }
  const events = JSON.parse(JSON.stringify(sourceEvents || []));
  const bounded = program.steps.some(
    (step) => step.primitive === "bounded_iterate_to_fixpoint",
  );
  const bound = bounded ? program.limits.maxIterations : 1;
  let selection = [];
  let projection = {};
  let matched = 0;
  let changed = 0;
  const matchedEventIds = new Set();
  for (let iteration = 1; iteration <= bound; iteration += 1) {
    const before = changed;
    for (const step of program.steps) {
      if (["sequential_compose", "bounded_iterate_to_fixpoint"].includes(step.primitive)) {
        continue;
      }
      if (step.primitive === "match") {
        selection = memoryProgramActiveIndices(events).filter((index) =>
          memoryProgramEventMatches(events[index], step.arguments),
        );
        projection = {};
        matched = Math.max(matched, selection.length);
        if (selection.length > program.limits.maxMatches) {
          return {
            programId: program.id,
            matched: selection.length,
            changed: 0,
            iterations: iteration,
            halt: "match_limit",
            maxMatches: program.limits.maxMatches,
            events: sourceEvents,
            matchedEventIds: [],
          };
        }
        selection.forEach((index) =>
          matchedEventIds.add(memoryProgramSourceId(events[index], index)),
        );
        continue;
      }
      if (step.primitive === "filter") {
        selection = memoryProgramFilter(events, selection, step.arguments);
      } else if (step.primitive === "map_matches") {
        projection = { ...step.arguments };
      } else if (step.primitive === "update") {
        changed += memoryProgramUpdate(events, selection, step.arguments);
      } else if (step.primitive === "create") {
        changed += memoryProgramCreate(events, selection, projection, step.arguments);
      } else if (step.primitive === "delete_with_retraction") {
        changed += memoryProgramRetract(events, selection, step.arguments);
      } else {
        return {
          programId: program.id,
          matched,
          changed,
          iterations: iteration,
          halt: "program_gap",
          primitive: step.primitive,
          events,
          matchedEventIds: [...matchedEventIds].sort(),
        };
      }
    }
    if (bounded && changed === before) {
      return {
        programId: program.id,
        matched,
        changed,
        iterations: iteration,
        halt: "fixpoint",
        events,
        matchedEventIds: [...matchedEventIds].sort(),
      };
    }
    if (!bounded) {
      return {
        programId: program.id,
        matched,
        changed,
        iterations: iteration,
        halt: "complete",
        events,
        matchedEventIds: [...matchedEventIds].sort(),
      };
    }
  }
  return {
    programId: program.id,
    matched,
    changed,
    iterations: bound,
    halt: "iteration_limit",
    maxIterations: bound,
    events,
    matchedEventIds: [...matchedEventIds].sort(),
  };
}

function memoryProgramExecutionLinks(outcome) {
  const lines = [
    "memory_program_execution",
    `  program ${memoryProgramLinoValue(outcome.programId)}`,
    `  matched ${outcome.matched}`,
    `  changed ${outcome.changed}`,
    `  iterations ${outcome.iterations}`,
    `  halt ${outcome.halt}`,
  ];
  if (outcome.halt === "match_limit") {
    lines.push(
      `  reason matched ${outcome.matched} exceeds max_matches ${outcome.maxMatches}`,
    );
  } else if (outcome.halt === "iteration_limit") {
    lines.push(`  reason max_iterations ${outcome.maxIterations} reached`);
  } else if (outcome.halt === "permission_denied") {
    lines.push(`  required ${memoryProgramLinoValue(outcome.required)}`);
    if (outcome.required === "destructive") {
      lines.push("  policy destructive_action_requires_confirmation");
    }
  } else if (outcome.halt === "program_gap") {
    lines.push(`  primitive ${memoryProgramLinoValue(outcome.primitive)}`);
  }
  for (const id of outcome.matchedEventIds || []) {
    lines.push(`  matched_event ${memoryProgramLinoValue(id)}`);
  }
  return lines.join("\n");
}

function memoryProgramFillResponse(intent, language, values = {}) {
  let response = answerFor(intent, language);
  for (const [name, value] of Object.entries(values)) {
    response = response.split(`{${name}}`).join(String(value));
  }
  return response;
}

function memoryProgramOutcomeResponse(outcome, language) {
  if (["complete", "fixpoint"].includes(outcome.halt)) {
    return memoryProgramFillResponse("memory_program_complete", language, {
      program: outcome.programId,
      matched: outcome.matched,
      changed: outcome.changed,
      halt: outcome.halt,
      iterations: outcome.iterations,
    });
  }
  if (outcome.halt === "match_limit") {
    return memoryProgramFillResponse("memory_program_match_limit", language, {
      matched: outcome.matched,
      max_matches: outcome.maxMatches,
    });
  }
  if (outcome.halt === "iteration_limit") {
    return memoryProgramFillResponse("memory_program_iteration_limit", language, {
      max_iterations: outcome.maxIterations,
    });
  }
  if (outcome.halt === "permission_denied" && outcome.required === "destructive") {
    return memoryProgramFillResponse("memory_program_destructive_refused", language);
  }
  if (outcome.halt === "permission_denied") {
    return memoryProgramFillResponse("memory_program_permission_refused", language, {
      required: outcome.required,
    });
  }
  return memoryProgramFillResponse("memory_program_interpreter_gap", language, {
    primitive: outcome.primitive,
  });
}

function memoryProgramOperation(sourceEvents, resultEvents) {
  const sourceLength = sourceEvents.length;
  const updates = [];
  for (let index = 0; index < sourceLength; index += 1) {
    const before = sourceEvents[index] || {};
    const after = resultEvents[index] || {};
    const fields = {};
    for (const name of ["kind", "content", "inputs", "outputs", "writeCount"]) {
      if (before[name] !== after[name]) fields[name] = after[name];
    }
    if (Object.keys(fields).length > 0) {
      updates.push({ id: before.id, fields });
    }
  }
  return {
    action: "program",
    updates,
    appends: resultEvents.slice(sourceLength),
  };
}

function tryMemoryProgram(prompt, events, language) {
  const compilation = compileMemoryProgramOnce(prompt);
  if (compilation.status !== "compiled") return null;
  const sourceEvents = Array.isArray(events) ? events : [];
  const outcome = executeMemoryProgramForWorker(compilation.program, sourceEvents);
  const compiledTrace = memoryProgramLinksNotation(compilation.program);
  const executionTrace = memoryProgramExecutionLinks(outcome);
  const answer = {
    intent: outcome.halt === "permission_denied" ? "memory_program_refused" : "memory_program",
    content: memoryProgramOutcomeResponse(outcome, language),
    confidence: 0.9,
    evidence: [
      `memory_program_compiled:${compiledTrace}`,
      `memory_program_execution:${executionTrace}`,
      "response:memory_program",
    ],
  };
  if (outcome.changed > 0) {
    answer.memoryOperation = memoryProgramOperation(sourceEvents, outcome.events);
  }
  return answer;
}

function tryMemoryProgramGap(prompt, language) {
  const compilation = compileMemoryProgramOnce(prompt);
  if (compilation.status !== "gap") return null;
  return {
    intent: "memory_program_gap",
    content: memoryProgramFillResponse("memory_program_compilation_gap", language, {
      gap: compilation.gap,
    }),
    confidence: 0.4,
    evidence: ["program_gap", compilation.gap, "response:memory_program_gap"],
  };
}
