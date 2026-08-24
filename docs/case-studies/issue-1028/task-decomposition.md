# Issue #1028 — recursive binary task tree

This is a complete full binary tree, not a flat list. Depth 0 is the root; depths 1–5 contain 2, 4, 8, 16, and 32 nodes respectively. The complete tree therefore contains exactly 63 task formulations (`1 + 2 + 4 + 8 + 16 + 32`).

The machine-readable canonical tree is `docs/case-studies/issue-1028/task-tree.json`. Every internal node has exactly two children, and every leaf is atomic and independently checkable.

## Tree

```text
R
├─ 1
│  ├─ 1.1
│  │  ├─ 1.1.1
│  │  │  ├─ 1.1.1.1
│  │  │  │  ├─ 1.1.1.1.1 (L01)
│  │  │  │  └─ 1.1.1.1.2 (L02)
│  │  │  └─ 1.1.1.2
│  │  │     ├─ 1.1.1.2.1 (L03)
│  │  │     └─ 1.1.1.2.2 (L04)
│  │  └─ 1.1.2
│  │     ├─ 1.1.2.1
│  │     │  ├─ 1.1.2.1.1 (L05)
│  │     │  └─ 1.1.2.1.2 (L06)
│  │     └─ 1.1.2.2
│  │        ├─ 1.1.2.2.1 (L07)
│  │        └─ 1.1.2.2.2 (L08)
│  └─ 1.2
│     ├─ 1.2.1
│     │  ├─ 1.2.1.1
│     │  │  ├─ 1.2.1.1.1 (L09)
│     │  │  └─ 1.2.1.1.2 (L10)
│     │  └─ 1.2.1.2
│     │     ├─ 1.2.1.2.1 (L11)
│     │     └─ 1.2.1.2.2 (L12)
│     └─ 1.2.2
│        ├─ 1.2.2.1
│        │  ├─ 1.2.2.1.1 (L13)
│        │  └─ 1.2.2.1.2 (L14)
│        └─ 1.2.2.2
│           ├─ 1.2.2.2.1 (L15)
│           └─ 1.2.2.2.2 (L16)
└─ 2
   ├─ 2.1
   │  ├─ 2.1.1
   │  │  ├─ 2.1.1.1
   │  │  │  ├─ 2.1.1.1.1 (L17)
   │  │  │  └─ 2.1.1.1.2 (L18)
   │  │  └─ 2.1.1.2
   │  │     ├─ 2.1.1.2.1 (L19)
   │  │     └─ 2.1.1.2.2 (L20)
   │  └─ 2.1.2
   │     ├─ 2.1.2.1
   │     │  ├─ 2.1.2.1.1 (L21)
   │     │  └─ 2.1.2.1.2 (L22)
   │     └─ 2.1.2.2
   │        ├─ 2.1.2.2.1 (L23)
   │        └─ 2.1.2.2.2 (L24)
   └─ 2.2
      ├─ 2.2.1
      │  ├─ 2.2.1.1
      │  │  ├─ 2.2.1.1.1 (L25)
      │  │  └─ 2.2.1.1.2 (L26)
      │  └─ 2.2.1.2
      │     ├─ 2.2.1.2.1 (L27)
      │     └─ 2.2.1.2.2 (L28)
      └─ 2.2.2
         ├─ 2.2.2.1
         │  ├─ 2.2.2.1.1 (L29)
         │  └─ 2.2.2.1.2 (L30)
         └─ 2.2.2.2
            ├─ 2.2.2.2.1 (L31)
            └─ 2.2.2.2.2 (L32)
```

## Level semantics

| Depth | Nodes | What is verified |
|---:|---:|---|
| 1 | 2 | the two top-level halves |
| 2 | 4 | each half split into two |
| 3 | 8 | each level-2 task split again |
| 4 | 16 | each level-3 task split again |
| 5 | 32 | atomic leaf tasks |

The Agent CLI ladder runs these levels independently in fresh temporary copies, starting with the 32 smallest leaves and moving upward. A failure at any level is a real failure and blocks the next level until the underlying capability is fixed and the failed node is rerun with a differently worded equivalent task.
