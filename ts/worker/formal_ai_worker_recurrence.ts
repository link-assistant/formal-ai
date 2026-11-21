function recurrenceFold(value) {
  return String(value || "")
    .toLowerCase()
    .normalize("NFD")
    .replace(/\p{M}/gu, "")
    .replace(/[^\p{L}\p{N}!]+/gu, " ")
    .trim();
}

function recurrenceExpressionNode(record) {
  const container = codingFindChild(record, "expression");
  return container && container.children.length === 1
    ? container.children[0]
    : null;
}

function renderRecurrenceExpression(node, functionName) {
  if (!node) return null;
  const rendered = node.children.map((child) =>
    renderRecurrenceExpression(child, functionName),
  );
  if (rendered.some((value) => value === null)) return null;
  if (node.name === "parameter") return node.value;
  if (node.name === "literal") return node.value;
  if (node.name === "recur" && rendered.length === 1) {
    const argument =
      rendered[0].startsWith("(") && rendered[0].endsWith(")")
        ? rendered[0].slice(1, -1)
        : rendered[0];
    return `${functionName}(${argument})`;
  }
  if (node.name === "conditional" && rendered.length === 3) {
    return `${rendered[1]} if ${rendered[0]} else ${rendered[2]}`;
  }
  const binary = {
    less_equal: "<=",
    equal: "==",
    add: "+",
    multiply: "*",
    subtract: "-",
  }[node.name];
  if (binary && rendered.length === 2) {
    return `(${rendered[0]} ${binary} ${rendered[1]})`;
  }
  if (node.name === "subtract_one" && rendered.length === 1) {
    return `${rendered[0]} - 1`;
  }
  return null;
}

function evaluateRecurrenceExpression(node, root, parameter, value, depth, memo = new Map()) {
  if (!node || depth > 512) return null;
  if (node.name === "parameter") return node.value === parameter ? value : null;
  if (node.name === "literal") {
    return /^-?\d{1,16}$/u.test(node.value) ? BigInt(node.value) : null;
  }
  if (node.name === "recur" && node.children.length === 1) {
    const argument = evaluateRecurrenceExpression(
      node.children[0],
      root,
      parameter,
      value,
      depth + 1,
      memo,
    );
    if (typeof argument !== "bigint" || argument < 0n || argument >= value) return null;
    if (!memo.has(argument)) {
      memo.set(argument, evaluateRecurrenceExpression(root, root, parameter, argument, depth + 1, memo));
    }
    return memo.get(argument);
  }
  if (node.name === "conditional" && node.children.length === 3) {
    const condition = evaluateRecurrenceExpression(
      node.children[0],
      root,
      parameter,
      value,
      depth + 1,
      memo,
    );
    if (typeof condition !== "boolean") return null;
    return evaluateRecurrenceExpression(
      condition ? node.children[1] : node.children[2],
      root,
      parameter,
      value,
      depth + 1,
      memo,
    );
  }
  const operands = node.children.map((child) =>
    evaluateRecurrenceExpression(child, root, parameter, value, depth + 1, memo),
  );
  if (operands.some((operand) => typeof operand !== "bigint")) return null;
  if (node.name === "less_equal" && operands.length === 2) {
    return operands[0] <= operands[1];
  }
  if (node.name === "equal" && operands.length === 2) {
    return operands[0] === operands[1];
  }
  if (node.name === "add" && operands.length === 2) {
    return operands[0] + operands[1];
  }
  if (node.name === "multiply" && operands.length === 2) {
    const product = operands[0] * operands[1];
    const limit = 1n << 126n;
    return product > -limit && product < limit ? product : null;
  }
  if (node.name === "subtract" && operands.length === 2) {
    return operands[0] - operands[1];
  }
  if (node.name === "subtract_one" && operands.length === 1) {
    return operands[0] - 1n;
  }
  return null;
}

function verifiedRecurrenceRecord(record) {
  const root = recurrenceExpressionNode(record);
  const parameter = codingChildValue(record, "parameter");
  const tests = record.children.filter((child) => child.name === "source_test");
  return (
    root &&
    parameter &&
    tests.length > 0 &&
    tests.every((test) => {
      const argument = Number(codingChildValue(test, "argument"));
      const expected = Number(codingChildValue(test, "expected"));
      return (
        Number.isInteger(argument) && argument >= 0 && argument <= 20
        && Number.isSafeInteger(expected)
        && evaluateRecurrenceExpression(root, root, parameter, BigInt(argument), 0) === BigInt(expected)
      );
    })
  );
}

function recurrenceObservation(prompt) {
  const match = String(prompt || "").match(/\b(\d+)(?:st|nd|rd|th)?\b/i);
  // Shared policy for a bounded demonstration when no observation was asked.
  const value = match ? Number(match[1]) : 10;
  return value >= 0 && value <= 20 ? value : null;
}
