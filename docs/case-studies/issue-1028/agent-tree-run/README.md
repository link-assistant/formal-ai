# Recursive Formal AI decomposition tree run

- requested depth: 5
- node filter: none
- selected nodes: 32
- failures: 0

The canonical decomposition is a complete binary tree: depth 0 has 1 node,
depth 1 has 2, depth 2 has 4, depth 3 has 8, depth 4 has 16, and depth 5 has 32.
Each selected node runs in a fresh temporary repository copy against the real
`@link-assistant/agent` CLI and a local `formal-ai serve --agent-mode`.

The `all` mode verifies the smallest atomic tasks first (32 leaves), then
16, 8, 4, 2, and finally the root, stopping on the first real failure so the
underlying capability can be repaired before larger composite tasks are tested.
