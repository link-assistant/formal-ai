// App hooks for document-level environment and context-panel layout.

import React from "react";
import {
  CONTEXT_PANEL_MIN_WIDTH, contextPanelMaxWidth, normalizeContextPanelWidth,
} from "./preferences.jsx";
import { i18nApi } from "./user-context.jsx";

const { useCallback, useEffect } = React;

// Keeps <html> lang/theme, the visual-viewport CSS variables and the
// i18n/colour-scheme re-render ticks in sync with the browser.
export function useDocumentEnvironment({
  setI18nRuntimeTick, uiLanguage, setContextPanelWidth, themePreference, setColorSchemeTick,
}) {
  useEffect(() => {
    if (typeof document === "undefined") return;
    document.documentElement.lang = uiLanguage;
    document.documentElement.dir = "ltr";
  }, [uiLanguage]);

  useEffect(() => {
    if (typeof document === "undefined") return;
    if (themePreference === "dark") {
      document.documentElement.setAttribute("data-theme", "dark");
    } else if (themePreference === "light") {
      document.documentElement.setAttribute("data-theme", "light");
    } else {
      document.documentElement.removeAttribute("data-theme");
    }
  }, [themePreference]);

  useEffect(() => {
    if (typeof window === "undefined" || typeof document === "undefined") {
      return undefined;
    }
    const root = document.documentElement;
    const updateViewport = () => {
      const visualViewport = window.visualViewport;
      const width =
        visualViewport && visualViewport.width
          ? visualViewport.width
          : window.innerWidth;
      const height =
        visualViewport && visualViewport.height
          ? visualViewport.height
          : window.innerHeight;
      const offsetLeft =
        visualViewport && visualViewport.offsetLeft
          ? visualViewport.offsetLeft
          : 0;
      const offsetTop =
        visualViewport && visualViewport.offsetTop
          ? visualViewport.offsetTop
          : 0;
      root.style.setProperty(
        "--formal-ai-viewport-width",
        `${Math.round(width)}px`,
      );
      root.style.setProperty(
        "--formal-ai-viewport-height",
        `${Math.round(height)}px`,
      );
      root.style.setProperty(
        "--formal-ai-viewport-offset-left",
        `${Math.round(offsetLeft)}px`,
      );
      root.style.setProperty(
        "--formal-ai-viewport-offset-top",
        `${Math.round(offsetTop)}px`,
      );
    };
    updateViewport();
    window.addEventListener("resize", updateViewport);
    window.addEventListener("orientationchange", updateViewport);
    if (window.visualViewport) {
      window.visualViewport.addEventListener("resize", updateViewport);
      window.visualViewport.addEventListener("scroll", updateViewport);
    }
    return () => {
      window.removeEventListener("resize", updateViewport);
      window.removeEventListener("orientationchange", updateViewport);
      if (window.visualViewport) {
        window.visualViewport.removeEventListener("resize", updateViewport);
        window.visualViewport.removeEventListener("scroll", updateViewport);
      }
    };
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return undefined;
    const clampContextPanel = () => {
      setContextPanelWidth((width) => normalizeContextPanelWidth(width));
    };
    window.addEventListener("resize", clampContextPanel);
    window.addEventListener("orientationchange", clampContextPanel);
    if (window.visualViewport) {
      window.visualViewport.addEventListener("resize", clampContextPanel);
    }
    return () => {
      window.removeEventListener("resize", clampContextPanel);
      window.removeEventListener("orientationchange", clampContextPanel);
      if (window.visualViewport) {
        window.visualViewport.removeEventListener("resize", clampContextPanel);
      }
    };
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return undefined;
    let cancelled = false;
    const update = () => {
      if (!cancelled) {
        setI18nRuntimeTick((value) => value + 1);
      }
    };
    window.addEventListener("formal-ai:i18n-ready", update);
    const api = i18nApi();
    if (api && api.ready && typeof api.ready.then === "function") {
      api.ready.then(update).catch(() => null);
    }
    return () => {
      cancelled = true;
      window.removeEventListener("formal-ai:i18n-ready", update);
    };
  }, []);

  useEffect(() => {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
      return undefined;
    }
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const update = () => setColorSchemeTick((value) => value + 1);
    if (typeof media.addEventListener === "function") {
      media.addEventListener("change", update);
      return () => media.removeEventListener("change", update);
    }
    if (typeof media.addListener === "function") {
      media.addListener(update);
      return () => media.removeListener(update);
    }
    return undefined;
  }, []);
}

// Pointer and keyboard resizing of the context (sidebar) panel.
export function useContextPanelResize({ contextPanelWidth, setContextPanelWidth }) {
  const handleContextResizePointerDown = useCallback((event) => {
    if (event.button !== 0 || typeof window === "undefined") return;
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = contextPanelWidth;
    const body = typeof document !== "undefined" ? document.body : null;
    const handlePointerMove = (moveEvent) => {
      const nextWidth = startWidth + moveEvent.clientX - startX;
      setContextPanelWidth(normalizeContextPanelWidth(nextWidth));
    };
    const stopResize = () => {
      if (body) {
        body.classList.remove("is-resizing-context");
      }
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", stopResize);
      window.removeEventListener("pointercancel", stopResize);
    };
    if (body) {
      body.classList.add("is-resizing-context");
    }
    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", stopResize);
    window.addEventListener("pointercancel", stopResize);
  }, [contextPanelWidth]);

  const handleContextResizeKeyDown = useCallback((event) => {
    const step = event.shiftKey ? 40 : 16;
    let nextWidth = null;
    if (event.key === "ArrowLeft") {
      nextWidth = contextPanelWidth - step;
    } else if (event.key === "ArrowRight") {
      nextWidth = contextPanelWidth + step;
    } else if (event.key === "Home") {
      nextWidth = CONTEXT_PANEL_MIN_WIDTH;
    } else if (event.key === "End") {
      nextWidth = contextPanelMaxWidth();
    }
    if (nextWidth === null) return;
    event.preventDefault();
    setContextPanelWidth(normalizeContextPanelWidth(nextWidth));
  }, [contextPanelWidth]);

  return { handleContextResizePointerDown, handleContextResizeKeyDown };
}
