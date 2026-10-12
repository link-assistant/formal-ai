import DOMPurify from "dompurify";
import { marked } from "marked";
import React from "react";
import { createRoot } from "react-dom/client";
// lino-i18n 0.3 splits its entries: the package root wires Node file loaders
// (`node:fs`), while `lino-i18n/browser` imports only platform APIs and adds
// URL catalog loading. The browser bundle takes the browser entry.
import { createI18n, loadCatalogs, parseLinoCatalogs } from "lino-i18n/browser";

window.React = React;
window.ReactDOM = { createRoot };
window.marked = marked;
window.DOMPurify = DOMPurify;
window.FormalAiVendor = {
  ...(window.FormalAiVendor || {}),
  LinoI18n: {
    createI18n,
    loadCatalogs,
    parseLinoCatalogs,
  },
};
