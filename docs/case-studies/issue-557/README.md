# Adaptive composer and skins

Drafted: composer actions share the text field surface at desktop, tablet and
mobile widths, with 44px targets. Existing flat/glass/contrast skins gain a Material
option. Glass opacity is persisted and ranges from 35–100%; reduced-transparency
and forced-color preferences have explicit fallbacks. Existing theme tokens carry
light/dark colors into the Material shape and elevation rules.

Research: [Chakra dark mode](https://chakra-ui.com/docs/styling/dark-mode) and
[Material UI dark mode](https://mui.com/material-ui/customization/dark-mode/)
both support light/dark designs. The app already uses Chakra. This draft keeps
that existing dependency and adds Material-inspired tokens; it does not claim
to install MUI or reproduce Apple's proprietary visual system. There is no
reliable universal “best rated” ranking in these publisher sources.

Requirements: adaptive field actions implemented; default/glass/material skin
choices implemented; glass slider implemented; theme support drafted. Visual
screenshots, mobile/desktop browser review, contrast measurement and a broader
UI-kit comparison remain unverified because no builds/tests were authorized.
