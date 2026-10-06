// Toolbar icon packs and the ToolbarIcon / ToolbarButton components.

import React from "react";
import { chakra } from "@chakra-ui/react";
import { normalizeToolbarIconPack } from "./preferences.jsx";

const { createElement: h } = React;

const TOOLBAR_ICON_FONT_NAMES = {
  fontawesome: {
    sourceCode: "fa-code",
    download: "fa-download",
    reportIssue: "fa-bug",
    exportMemory: "fa-file-export",
    importMemory: "fa-file-import",
    resetMemory: "fa-broom",
    diagnostics: "fa-flask-vial",
    chat: "fa-comment-dots",
    agent: "fa-robot",
    demo: "fa-clapperboard",
    attachFiles: "fa-paperclip",
    isolateSection: "fa-up-right-and-down-left-from-center",
  },
  "material-symbols": {
    sourceCode: "code",
    download: "download",
    reportIssue: "bug_report",
    exportMemory: "upload_file",
    importMemory: "file_download",
    resetMemory: "cleaning_services",
    diagnostics: "science",
    chat: "chat_bubble",
    agent: "smart_toy",
    demo: "movie",
    attachFiles: "attach_file",
    isolateSection: "open_in_full",
  },
  "bootstrap-icons": {
    sourceCode: "bi-code-slash",
    download: "bi-download",
    reportIssue: "bi-bug",
    exportMemory: "bi-file-earmark-arrow-up",
    importMemory: "bi-file-earmark-arrow-down",
    resetMemory: "bi-eraser",
    diagnostics: "bi-flask",
    chat: "bi-chat-dots",
    agent: "bi-robot",
    demo: "bi-play-btn",
    attachFiles: "bi-paperclip",
    isolateSection: "bi-arrows-fullscreen",
  },
  ionicons: {
    sourceCode: "code-slash-outline",
    download: "download-outline",
    reportIssue: "bug-outline",
    exportMemory: "cloud-upload-outline",
    importMemory: "cloud-download-outline",
    resetMemory: "brush-outline",
    diagnostics: "flask-outline",
    chat: "chatbubble-ellipses-outline",
    agent: "hardware-chip-outline",
    demo: "videocam-outline",
    attachFiles: "attach-outline",
    isolateSection: "expand-outline",
  },
  "remix-icon": {
    sourceCode: "ri-code-s-slash-line",
    download: "ri-download-line",
    reportIssue: "ri-bug-line",
    exportMemory: "ri-file-upload-line",
    importMemory: "ri-file-download-line",
    resetMemory: "ri-brush-3-line",
    diagnostics: "ri-flask-line",
    chat: "ri-chat-3-line",
    agent: "ri-robot-2-line",
    demo: "ri-movie-line",
    attachFiles: "ri-attachment-line",
    isolateSection: "ri-fullscreen-line",
  },
  "tabler-icons": {
    sourceCode: "IconCode",
    download: "IconDownload",
    reportIssue: "IconBug",
    exportMemory: "IconFileExport",
    importMemory: "IconFileImport",
    resetMemory: "IconEraser",
    diagnostics: "IconFlask",
    chat: "IconMessageCircle",
    agent: "IconRobot",
    demo: "IconMovie",
    attachFiles: "IconPaperclip",
    isolateSection: "IconArrowsMaximize",
  },
  names: {
    sourceCode: "Code",
    download: "Download",
    reportIssue: "Bug",
    exportMemory: "Export",
    importMemory: "Import",
    resetMemory: "Reset",
    diagnostics: "Diagnostics",
    chat: "Chat",
    agent: "Agent",
    demo: "Demo",
    attachFiles: "Attach",
    isolateSection: "Only",
  },
};

const TOOLBAR_ICON_SHORT_NAMES = {
  sourceCode: "Code",
  download: "Down",
  reportIssue: "Bug",
  exportMemory: "Out",
  importMemory: "In",
  resetMemory: "Clear",
  diagnostics: "Diag",
  chat: "Chat",
  agent: "Agent",
  demo: "Demo",
  attachFiles: "File",
  isolateSection: "One",
};

const TOOLBAR_ICON_SHAPES = {
  sourceCode: [
    ["path", { d: "M8.5 8.5 5 12l3.5 3.5" }],
    ["path", { d: "m15.5 8.5 3.5 3.5-3.5 3.5" }],
    ["path", { d: "m13.5 6-3 12" }],
  ],
  download: [
    ["path", { d: "M12 5v10" }],
    ["path", { d: "m8 11 4 4 4-4" }],
    ["path", { d: "M5 19h14" }],
  ],
  reportIssue: [
    ["path", { d: "M8 9h8v6a4 4 0 0 1-8 0V9Z" }],
    ["path", { d: "M9 9 7 6" }],
    ["path", { d: "m15 9 2-3" }],
    ["path", { d: "M12 8V5" }],
    ["path", { d: "M6 13H3.5" }],
    ["path", { d: "M20.5 13H18" }],
    ["path", { d: "M7 18l-2 2" }],
    ["path", { d: "m17 18 2 2" }],
    ["path", { d: "M10 13h.01" }],
    ["path", { d: "M14 13h.01" }],
  ],
  exportMemory: [
    ["path", { d: "M6 3h8l4 4v14H6V3Z" }],
    ["path", { d: "M14 3v5h4" }],
    ["path", { d: "M12 17V9" }],
    ["path", { d: "m8.5 12.5 3.5-3.5 3.5 3.5" }],
  ],
  importMemory: [
    ["path", { d: "M6 3h8l4 4v14H6V3Z" }],
    ["path", { d: "M14 3v5h4" }],
    ["path", { d: "M12 9v8" }],
    ["path", { d: "m8.5 13.5 3.5 3.5 3.5-3.5" }],
  ],
  resetMemory: [
    ["path", { d: "m15 4-7 7" }],
    ["path", { d: "m7 12 5 5" }],
    ["path", { d: "m5 14 5 5" }],
    ["path", { d: "m9 10 5 5" }],
    ["path", { d: "M4 20h16" }],
  ],
  diagnostics: [
    ["path", { d: "M10 4h4" }],
    ["path", { d: "M11 4v5l-5 8a3 3 0 0 0 2.5 4h7a3 3 0 0 0 2.5-4l-5-8V4" }],
    ["path", { d: "M8 16h8" }],
  ],
  chat: [
    ["path", { d: "M5 6h14v9H9l-4 4V6Z" }],
    ["path", { d: "M8.5 10.5h7" }],
    ["path", { d: "M8.5 13h4" }],
  ],
  agent: [
    ["path", { d: "M8 9h8v8H8V9Z" }],
    ["path", { d: "M12 9V5" }],
    ["path", { d: "M9.5 5h5" }],
    ["path", { d: "M6 12H4" }],
    ["path", { d: "M20 12h-2" }],
    ["path", { d: "M10 12h.01" }],
    ["path", { d: "M14 12h.01" }],
    ["path", { d: "M10 15h4" }],
  ],
  demo: [
    ["path", { d: "M4 7h16v12H4V7Z" }],
    ["path", { d: "M4 11h16" }],
    ["path", { d: "m7 7 2-4" }],
    ["path", { d: "m12 7 2-4" }],
    ["path", { d: "m17 7 2-4" }],
    ["path", { d: "m10 14 4 2-4 2v-4Z" }],
  ],
  attachFiles: [
    ["path", { d: "m8 12 5.5-5.5a3 3 0 0 1 4.25 4.25l-7.25 7.25a5 5 0 0 1-7.07-7.07L10 4.36" }],
    ["path", { d: "m9.5 14.5 6-6" }],
  ],
  isolateSection: [
    ["path", { d: "M8 3H3v5" }],
    ["path", { d: "M3 3l6 6" }],
    ["path", { d: "M16 3h5v5" }],
    ["path", { d: "M21 3l-6 6" }],
    ["path", { d: "M8 21H3v-5" }],
    ["path", { d: "M3 21l6-6" }],
    ["path", { d: "M16 21h5v-5" }],
    ["path", { d: "M21 21l-6-6" }],
  ],
};

function toolbarIconFontName(action, pack) {
  const normalizedPack = normalizeToolbarIconPack(pack);
  return (
    TOOLBAR_ICON_FONT_NAMES[normalizedPack]?.[action] ||
    TOOLBAR_ICON_FONT_NAMES.fontawesome[action] ||
    action
  );
}

function toolbarIconFontClass(action, pack) {
  const normalizedPack = normalizeToolbarIconPack(pack);
  const name = toolbarIconFontName(action, normalizedPack);
  if (normalizedPack === "fontawesome") return `fa-solid ${name}`;
  if (normalizedPack === "material-symbols") return `material-symbols-outlined ${name}`;
  if (normalizedPack === "bootstrap-icons") return `bi ${name}`;
  if (normalizedPack === "remix-icon") return `ri ${name}`;
  if (normalizedPack === "tabler-icons") return `ti ${name}`;
  return name;
}

export function ToolbarIcon({ action, pack, className = "btn-icon" }) {
  const normalizedPack = normalizeToolbarIconPack(pack);
  const fontName = toolbarIconFontName(action, normalizedPack);
  const fontClass = toolbarIconFontClass(action, normalizedPack);
  const baseClass = `${className} toolbar-icon icon-pack-${normalizedPack}`;
  if (normalizedPack === "names") {
    return <span className={`${baseClass} toolbar-icon-name`} aria-hidden="true" data-icon-pack={normalizedPack} data-icon-font-name={fontName} data-icon-font-class={fontClass}>{TOOLBAR_ICON_SHORT_NAMES[action] || fontName}</span>;
  }
  const shape = TOOLBAR_ICON_SHAPES[action] || TOOLBAR_ICON_SHAPES.chat;
  return <span className={baseClass} aria-hidden="true" data-icon-pack={normalizedPack} data-icon-font-name={fontName} data-icon-font-class={fontClass}><svg className="toolbar-icon-svg" viewBox="0 0 24 24" focusable="false">{shape.map(([tag, attrs], index) => {
      // The SVG primitive tag (path/circle/rect/…) is data-driven, so alias it
      // to a capitalised name: JSX treats lowercase element names as literal
      // string tags, only Capitalised names resolve to a variable. <Shape …/>
      // compiles to h(Shape, {…}) === h(tag, {…}).
      const Shape = tag;
      return <Shape {...attrs} key={`${action}-${index}`} />;
    })}</svg></span>;
}

// Reusable top-menu control (issue #550). Every topbar button/link was hand
// written as the same icon + localized `.btn-label` span pair, so they drifted:
// some gained a hover/focus treatment and some didn't (the P5 "only some
// buttons react to hover" defect). Routing them all through ONE component means
// they share the same markup contract — `.btn-label`, the icon, and the
// className that the single CSS hover/focus rule targets — so a new control
// cannot silently miss the shared affordance. Renders an <a> when `href` is a
// string, otherwise a <button type="button">. Localized text stays at the call
// site as `label`/`title`/`ariaLabel` props (still real `t(...)` calls, so the
// hardcoded-UI check keeps passing); this component never embeds prose.
// Pairs with the `--fa-control-*` design tokens in styles.css (the shared
// hover/active/focus treatment); see docs/case-studies/issue-550 for the full
// rationale (M2 reusable-component requirement + the Chakra/CSP ADR).
export function ToolbarButton({
  className,
  label,
  icon,
  iconPack,
  href,
  onClick,
  title,
  ariaLabel,
  testId,
  menuPriority,
  target,
  rel,
  type = "button",
  extraProps = null,
  children = null,
}) {
  const isLink = typeof href === "string";
  const props = { className };
  if (title !== undefined) props.title = title;
  if (ariaLabel !== undefined) props["aria-label"] = ariaLabel;
  if (testId !== undefined) props["data-testid"] = testId;
  if (menuPriority !== undefined) props["data-menu-priority"] = menuPriority;
  if (isLink) {
    props.href = href;
    if (target !== undefined) props.target = target;
    if (rel !== undefined) props.rel = rel;
  } else {
    props.type = type;
    if (onClick) props.onClick = onClick;
  }
  // Caller-supplied attributes (aria-pressed, role, data-mode, key, …) for the
  // segmented/toggle controls. Merged last so a control can extend the shared
  // contract without forking it.
  if (extraProps) Object.assign(props, extraProps);
  // Render through the Chakra styled factory (chakra.a / chakra.button). These
  // are the low-level primitives — they carry no component recipe, so no Chakra
  // styling is imposed; the element keeps its className and styles.css stays
  // authoritative (preflight is off). This is the safe first step of the
  // h() → JSX + Chakra migration: identical DOM and computed styles.
  const Tag = isLink ? chakra.a : chakra.button;
  return (
    <Tag {...props}>
      {icon ? <ToolbarIcon action={icon} pack={iconPack} /> : null}
      {label !== undefined && label !== null ? (
        <chakra.span className="btn-label">{label}</chakra.span>
      ) : null}
      {children}
    </Tag>
  );
}
