# Issue 476 Online Research

Collected: 2026-06-27

## Sources Checked

- WAI-ARIA Authoring Practices Guide, Accordion Pattern: https://www.w3.org/WAI/ARIA/apg/patterns/accordion/
- MDN, `aria-expanded`: https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-expanded
- Lucide, `maximize` icon: https://lucide.dev/icons/maximize
- Material Symbols, `open_in_full` icon: https://fonts.google.com/icons?selected=Material+Symbols+Outlined:open_in_full

## Notes

- APG treats accordion headers as controls for showing and hiding associated content. It also explicitly allows persistent adjacent controls next to the header, which matches a right-side "expand only" action.
- APG and MDN both point to keeping `aria-expanded` on the focusable control that toggles the section state.
- Fullscreen/maximize/open-in-full iconography is a common visual metaphor for expanding a region. The app already has a local `ToolbarIcon` abstraction, so adding an isolate/fullscreen-like action there is lower risk than introducing a new icon library.
