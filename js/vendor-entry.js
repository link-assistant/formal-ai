import DOMPurify from "dompurify";
import { marked } from "marked";
import React from "react";
import { createRoot } from "react-dom/client";
import { createI18n, parseLinoCatalogs } from "lino-i18n";

window.React = React;
window.ReactDOM = { createRoot };
window.marked = marked;
window.DOMPurify = DOMPurify;
window.FormalAiVendor = {
  ...(window.FormalAiVendor || {}),
  LinoI18n: {
    createI18n,
    parseLinoCatalogs,
  },
};
