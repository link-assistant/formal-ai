import React from "react";
const { createElement: h } = React;

export function MenuGlyph({ open }) {
  return <span className={`btn-icon menu-icon ${open ? "menu-icon-close" : "menu-icon-hamburger"}`} aria-hidden="true" />;
}

export function SidebarToggleGlyph({ collapsed }) {
  return <span className={`btn-icon sidebar-toggle-icon ${collapsed ? "sidebar-toggle-icon-expand" : "sidebar-toggle-icon-collapse"}`} aria-hidden="true">{collapsed ? "▶" : "◀"}</span>;
}
