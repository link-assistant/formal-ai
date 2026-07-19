# Issue #557 Online Research Snapshot

Captured on 2026-07-08 for the adaptive UI/skin work.

React Bits follow-up research was captured on 2026-07-10; see
[`react-bits-research.md`](react-bits-research.md).

## Official Design References

- Material Design components: https://m3.material.io/components
- MUI Material UI getting started: https://mui.com/material-ui/getting-started/
- Chakra UI docs: https://chakra-ui.com/
- Ant Design docs: https://ant.design/
- Apple Human Interface Guidelines - Materials: https://developer.apple.com/design/human-interface-guidelines/materials
- Apple Liquid Glass overview: https://developer.apple.com/documentation/technologyoverviews/liquid-glass

## GitHub Popularity Snapshot

Raw JSON is saved beside this file as `ui-kit-*.json`.

| Repository | Stars | Forks |
|---|---:|---:|
| shadcn-ui/ui | 118,465 | 9,288 |
| ant-design/ant-design | 98,610 | 54,645 |
| mui/material-ui | 98,555 | 32,591 |
| chakra-ui/chakra-ui | 40,485 | 3,623 |
| mantinedev/mantine | 31,404 | 2,329 |
| DavidHDev/react-bits (2026-07-10) | 43,104 | 2,007 |

## Product Takeaways

- Keep the adaptive composer as a first-party formal-ai control instead of importing a full UI kit for one surface.
- Model skins as tokens and CSS classes so skins remain cheap to switch and easy to test.
- Treat Material as tonal surfaces plus subtle elevation.
- Treat Glass as a translucency material with an explicit opacity control.
- Keep settings compact and direct, matching enterprise/productivity UI conventions.
- Treat React Bits as a copy-paste component/design reference, not another UI
  framework: its own app depends on Chakra UI and its components ship as
  React/CSS or React/Tailwind source variants.
- Adopt its progressive glass practices (semantic controls, decorative layers,
  responsive sizing, capability fallback) without importing its WebGL hero
  modes into persistent productivity controls.
