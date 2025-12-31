const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const os = require('os');
const { exec } = require('child_process');

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (require('electron-squirrel-startup')) {
  app.quit();
}

let mainWindow;

const createWindow = () => {
  mainWindow = new BrowserWindow({
    width: 900,
    height: 600,
    frame: false, // Frameless window
    titleBarStyle: 'hidden',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      nodeIntegration: false,
      contextIsolation: true,
    },
    backgroundColor: '#f5f7fa',
    resizable: false, // Keep fixed size like the screenshot implies
  });

  if (process.env.NODE_ENV === 'development') {
    mainWindow.loadURL('http://localhost:5173');
    // mainWindow.webContents.openDevTools();
  } else {
    mainWindow.loadFile(path.join(__dirname, '../dist/index.html'));
  }
};

app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

// --- IPC Handlers ---

ipcMain.handle('get-cpu-info', () => {
  const cpus = os.cpus();
  // Simplify model name
  const model = cpus[0].model.trim();
  return {
    model: model,
    cores: cpus.length,
    speed: cpus[0].speed
  };
});

ipcMain.handle('get-processes', async () => {
  return new Promise((resolve, reject) => {
    const isWin = process.platform === 'win32';
    // Mac: ps -ax -o pid,comm
    // Win: tasklist
    const cmd = isWin 
      ? 'tasklist /FO CSV /NH' 
      : 'ps -ax -o pid,comm';

    exec(cmd, (error, stdout, stderr) => {
      if (error) {
        console.error(`exec error: ${error}`);
        resolve([]);
        return;
      }

      const processes = [];
      const lines = stdout.split('\n');

      if (isWin) {
        lines.forEach(line => {
          if (!line.trim()) return;
          const parts = line.split('","');
          if (parts.length > 1) {
            const name = parts[0].replace('"', '');
            const pid = parts[1].replace('"', '');
            processes.push({ pid: parseInt(pid), name });
          }
        });
      } else {
        lines.forEach(line => {
          const trimmed = line.trim();
          if (!trimmed || trimmed.includes('PID COMMAND')) return;
          const spaceIdx = trimmed.indexOf(' ');
          if (spaceIdx > 0) {
            const pid = trimmed.substring(0, spaceIdx);
            const name = trimmed.substring(spaceIdx + 1);
            const shortName = path.basename(name); // Use basename for cleaner UI
            processes.push({ pid: parseInt(pid), name: shortName });
          }
        });
      }
      
      processes.sort((a, b) => a.name.localeCompare(b.name));
      resolve(processes);
    });
  });
});

ipcMain.handle('set-affinity', (event, { pid, coreMask }) => {
  const isWin = process.platform === 'win32';
  
  if (isWin) {
    const cmd = `powershell -Command "$Process = Get-Process -Id ${pid}; $Process.ProcessorAffinity = ${coreMask}"`;
    console.log(`Executing: ${cmd}`);
    
    return new Promise((resolve) => {
      exec(cmd, (error) => {
        if (error) {
          console.error('Affinity Error:', error);
          resolve({ success: false, error: error.message });
        } else {
          resolve({ success: true });
        }
      });
    });
  } else {
    console.log(`[Simulation] Setting affinity for PID ${pid} to mask ${coreMask}`);
    return Promise.resolve({ success: true, message: "Simulated on macOS" });
  }
});

ipcMain.on('window-minimize', () => mainWindow?.minimize());
ipcMain.on('window-close', () => mainWindow?.close());
