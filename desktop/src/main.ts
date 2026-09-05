import { app, BrowserWindow, dialog, ipcMain } from 'electron';
import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';

import { registerApiHandlers } from './api';

let mainWindow: BrowserWindow | null = null;

function analyzerBinaryPath(): string {
  const binaryName = process.platform === 'win32' ? 'akron-analyzer.exe' : 'akron-analyzer';
  const packagedPath = join(process.resourcesPath, 'akron-runtime', binaryName);
  if (existsSync(packagedPath)) {
    return packagedPath;
  }

  return join(app.getAppPath(), '..', 'target', 'release', binaryName);
}

function createWindow(): void {
  mainWindow = new BrowserWindow({
    width: 1180,
    height: 760,
    minWidth: 900,
    minHeight: 600,
    show: false,
    backgroundColor: '#0b0d12',
    webPreferences: {
      preload: join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });

  void mainWindow.loadFile(join(app.getAppPath(), 'dist-renderer', 'index.html'));
  mainWindow.once('ready-to-show', () => mainWindow?.show());
  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

ipcMain.handle('dialog:pick-game-folder', async () => {
  const options = {
    title: 'Select a game folder',
    properties: ['openDirectory'] as const,
  };

  const result = mainWindow
    ? await dialog.showOpenDialog(mainWindow, options)
    : await dialog.showOpenDialog(options);

  return result.canceled ? null : (result.filePaths[0] ?? null);
});

ipcMain.handle('analyzer:analyze', async (_event, gamePath: unknown) => {
  if (typeof gamePath !== 'string' || gamePath.length === 0) {
    throw new Error('A game directory is required.');
  }

  const binary = analyzerBinaryPath();
  if (!existsSync(binary)) {
    throw new Error(`Akron Analyzer binary was not found: ${binary}`);
  }

  return await new Promise<unknown>((resolve, reject) => {
    const child = spawn(binary, [gamePath], {
      cwd: app.getAppPath(),
      stdio: ['ignore', 'pipe', 'pipe'],
      windowsHide: true,
    });

    let stdout = '';
    let stderr = '';

    child.stdout.setEncoding('utf8');
    child.stderr.setEncoding('utf8');
    child.stdout.on('data', (chunk: string) => {
      stdout += chunk;
    });
    child.stderr.on('data', (chunk: string) => {
      stderr += chunk;
    });
    child.once('error', (error) => {
      reject(new Error(`Failed to start Analyzer: ${error.message}`));
    });
    child.once('close', (code, signal) => {
      if (code !== 0) {
        const detail =
          stderr.trim() ||
          `Analyzer exited with code ${code ?? 'unknown'}${signal ? ` (${signal})` : ''}.`;
        reject(new Error(detail));
        return;
      }

      try {
        resolve(JSON.parse(stdout));
      } catch {
        reject(new Error('Analyzer returned invalid JSON.'));
      }
    });
  });
});

registerApiHandlers();

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
