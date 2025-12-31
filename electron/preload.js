const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electron', {
  getCpuInfo: () => ipcRenderer.invoke('get-cpu-info'),
  getProcesses: () => ipcRenderer.invoke('get-processes'),
  setAffinity: (pid, coreMask) => ipcRenderer.invoke('set-affinity', { pid, coreMask }),
  minimize: () => ipcRenderer.send('window-minimize'),
  close: () => ipcRenderer.send('window-close'),
});
