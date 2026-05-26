"use strict";

const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("FormalAiDesktop", {
  getStatus: () => ipcRenderer.invoke("formalAiDesktop:getStatus"),
  openExternal: (url) => ipcRenderer.invoke("formalAiDesktop:openExternal", url),
});
