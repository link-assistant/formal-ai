"use strict";

const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("FormalAiDesktop", {
  getStatus: () => ipcRenderer.invoke("formalAiDesktop:getStatus"),
  openExternal: (url) => ipcRenderer.invoke("formalAiDesktop:openExternal", url),
  // R5d: drive the default-deny tool-call gate and route permitted tools through
  // the local process / Docker sandbox.
  setToolGrants: (grants) => ipcRenderer.invoke("formalAiDesktop:setToolGrants", grants),
  invokeTool: (request) => ipcRenderer.invoke("formalAiDesktop:invokeTool", request),
  // R5c: reconcile browser (IndexedDB) memory with the native store.
  syncMemory: (payload) => ipcRenderer.invoke("formalAiDesktop:syncMemory", payload),
});
