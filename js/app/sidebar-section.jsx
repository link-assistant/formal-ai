// Collapsible sidebar section used by every context-panel group.

import React from "react";
import { ToolbarIcon } from "./toolbar-icons.jsx";

const { createElement: h } = React;

// Issue #27: a VS Code-style collapsible sidebar section. When `collapsed` is
// false the section participates in the equal-share flex layout and scrolls
// independently; when true only the header remains visible.
export const SIDEBAR_SECTION_TEST_IDS = [
  "drawer-menu-actions",
  "sidebar-desktop",
  "sidebar-services",
  "sidebar-conversations",
  "sidebar-settings",
  "sidebar-prompts",
  "sidebar-tools",
  "sidebar-trace",
];

export function CollapsibleSection({
  title,
  collapsed,
  onToggle,
  testId,
  className = "",
  bodyClassName = "",
  expandOnlyLabel,
  expandOnlyTitle,
  iconPack,
  children,
}) {
  const sectionClassName = [
    "sidebar-section",
    collapsed ? "is-collapsed" : "is-expanded",
    className,
  ]
    .filter(Boolean)
    .join(" ");
  const sectionBodyClassName = ["sidebar-section-body", bodyClassName]
    .filter(Boolean)
    .join(" ");
  const isolateLabel = expandOnlyLabel || title;
  const isolateTitle = expandOnlyTitle || isolateLabel;
  const handleHeaderClick = (event) => {
    const target = event.target;
    if (
      target &&
      typeof target.closest === "function" &&
      target.closest("[data-sidebar-section-action]")
    ) {
      return;
    }
    if (typeof onToggle === "function") onToggle();
  };
  const handleToggleClick = (event) => {
    event.stopPropagation();
    if (typeof onToggle === "function") onToggle();
  };
  return <section className={sectionClassName} data-testid={testId} data-collapsed={collapsed ? "true" : "false"}><div className="sidebar-section-header" onClick={handleHeaderClick}><button type="button" className="sidebar-section-toggle" aria-expanded={collapsed ? "false" : "true"} onClick={handleToggleClick}><span className="sidebar-section-caret" aria-hidden="true">{collapsed ? "▶" : "▼"}</span><h2>{title}</h2></button><button type="button" className="sidebar-section-isolate" data-testid="sidebar-section-isolate" data-sidebar-section-action="isolate" aria-label={isolateLabel} title={isolateTitle}><ToolbarIcon action="isolateSection" pack={iconPack} /></button></div>{collapsed ? null : <div className={sectionBodyClassName}>{children}</div>}</section>;
}
