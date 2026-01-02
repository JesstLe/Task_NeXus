const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electron', {
  getCpuInfo: () => ipcRenderer.invoke('get-cpu-info'),
  getProcesses: () => ipcRenderer.invoke('get-processes'),
  setAffinity: (pid, coreMask, mode) => ipcRenderer.invoke('set-affinity', pid, coreMask, mode),
  minimize: () => ipcRenderer.send('window-minimize'),
  close: () => ipcRenderer.send('window-close'),
  getSettings: () => ipcRenderer.invoke('get-settings'),
  setSetting: (key, value) => ipcRenderer.invoke('set-setting', key, value),
  addProfile: (profile) => ipcRenderer.invoke('add-profile', profile),
  removeProfile: (name) => ipcRenderer.invoke('remove-profile', name),
  getProfiles: () => ipcRenderer.invoke('get-profiles'),
});
