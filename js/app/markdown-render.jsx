// Markdown rendering, external-link hardening and code-block copy buttons.

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function isHttpExternalLink(href) {
  try {
    const url = new URL(href, window.location.href);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch (_error) {
    return /^https?:\/\//i.test(String(href || ""));
  }
}

function enhanceMarkdownLinks(html) {
  if (typeof document === "undefined") return html;
  const template = document.createElement("template");
  template.innerHTML = html;
  template.content.querySelectorAll("a[href]").forEach((anchor) => {
    const href = anchor.getAttribute("href") || "";
    if (!isHttpExternalLink(href)) return;
    anchor.setAttribute("target", "_blank");
    anchor.setAttribute("rel", "noopener noreferrer");
    anchor.classList.add("external-link");
    if (!anchor.querySelector(".external-link-icon")) {
      anchor.appendChild(document.createTextNode(" "));
      const icon = document.createElement("span");
      icon.className = "external-link-icon";
      icon.setAttribute("aria-hidden", "true");
      anchor.appendChild(icon);
    }
  });
  return template.innerHTML;
}

export function markdownHtml(value) {
  const text = String(value ?? "");
  if (window.marked && window.DOMPurify) {
    const html = window.marked.parse(text, {
      breaks: true,
      gfm: true,
    });
    return { __html: enhanceMarkdownLinks(window.DOMPurify.sanitize(html)) };
  }

  return { __html: escapeHtml(text).replaceAll("\n", "<br>") };
}

// Issue #330: copy helper shared by the per-code-block and per-message copy
// buttons. Prefers the async Clipboard API and falls back to a hidden textarea
// + execCommand so the feature still works in the Playwright/file:// contexts
// where the Clipboard API may be unavailable or permission-gated.
export async function copyTextToClipboard(text) {
  const value = String(text ?? "");
  if (
    typeof navigator !== "undefined" &&
    navigator.clipboard &&
    typeof navigator.clipboard.writeText === "function"
  ) {
    try {
      await navigator.clipboard.writeText(value);
      return true;
    } catch (_error) {
      // Fall through to the legacy path below.
    }
  }
  if (typeof document === "undefined") return false;
  try {
    const textarea = document.createElement("textarea");
    textarea.value = value;
    textarea.setAttribute("readonly", "");
    textarea.style.position = "fixed";
    textarea.style.top = "-1000px";
    textarea.style.opacity = "0";
    document.body.appendChild(textarea);
    textarea.select();
    const ok = document.execCommand("copy");
    document.body.removeChild(textarea);
    return ok;
  } catch (_error) {
    return false;
  }
}

// Flash a transient "Copied!" label on a button, then restore the original.
function flashCopied(button, copiedLabel, restoreLabel) {
  if (!button) return;
  button.classList.add("is-copied");
  button.setAttribute("data-copied", "true");
  const labelNode = button.querySelector(".copy-button-label") || button;
  labelNode.textContent = copiedLabel;
  if (button._copyResetTimer) {
    clearTimeout(button._copyResetTimer);
  }
  button._copyResetTimer = setTimeout(() => {
    button.classList.remove("is-copied");
    button.removeAttribute("data-copied");
    labelNode.textContent = restoreLabel;
    button._copyResetTimer = null;
  }, 1600);
}

// Issue #330: progressively enhance the code fences rendered by marked. Each
// `<pre><code class="language-xxx">` is syntax-highlighted in place and wrapped
// in a `.code-block` shell carrying a language label and a per-block copy
// button. The function is idempotent so it can run on every effect pass without
// double-wrapping existing blocks.
export function enhanceCodeBlocks(root, t) {
  if (!root || typeof document === "undefined") return;
  const highlighter =
    typeof window !== "undefined" ? window.FormalAiHighlight : null;
  const copyLabel = t ? t("message.copyCode") : "Copy";
  const copiedLabel = t ? t("message.copyCodeDone") : "Copied!";
  const copyTitle = t ? t("message.copyCodeTitle") : copyLabel;

  const blocks = root.querySelectorAll("pre > code");
  blocks.forEach((code) => {
    const pre = code.parentElement;
    if (!pre || pre.parentElement?.classList.contains("code-block")) {
      return; // already enhanced
    }

    const rawCode = code.textContent ?? "";
    const className = code.getAttribute("class") || "";
    const match = /language-([\w+#-]+)/i.exec(className);
    const requested = match ? match[1] : "";

    if (highlighter && typeof highlighter.highlight === "function") {
      const { value, language } = highlighter.highlight(rawCode, requested);
      code.innerHTML = value;
      code.classList.add("hljs");
      if (language) {
        code.setAttribute("data-language", language);
      }
    }

    const wrapper = document.createElement("div");
    wrapper.className = "code-block";

    const header = document.createElement("div");
    header.className = "code-block-header";

    const langLabel = document.createElement("span");
    langLabel.className = "code-block-lang";
    const resolved =
      highlighter && typeof highlighter.resolveLanguage === "function"
        ? highlighter.resolveLanguage(requested)
        : null;
    langLabel.textContent = (resolved || requested || "text").toLowerCase();

    const button = document.createElement("button");
    button.type = "button";
    button.className = "code-copy-button";
    button.setAttribute("data-testid", "code-copy-button");
    button.setAttribute("aria-label", copyTitle);
    button.setAttribute("title", copyTitle);
    const buttonLabel = document.createElement("span");
    buttonLabel.className = "copy-button-label";
    buttonLabel.textContent = copyLabel;
    button.appendChild(buttonLabel);
    button.addEventListener("click", async () => {
      const ok = await copyTextToClipboard(rawCode);
      if (ok) {
        flashCopied(button, copiedLabel, copyLabel);
      }
    });

    header.appendChild(langLabel);
    header.appendChild(button);

    pre.parentElement.insertBefore(wrapper, pre);
    wrapper.appendChild(header);
    wrapper.appendChild(pre);
  });
}
