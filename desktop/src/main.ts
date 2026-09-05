import { app, BrowserWindow, dialog, ipcMain, type OpenDialogOptions } from 'electron';
import { appendFileSync, existsSync, mkdirSync } from 'node:fs';
import { spawn } from 'node:child_process';
import { join } from 'node:path';

import { registerApiHandlers } from './api';

const startupLogDirectory = join(process.env.HOME ?? process.env.USERPROFILE ?? '.', 'Library', 'Logs', 'Akron');
const startupLogPath = join(startupLogDirectory, 'startup.log');

function logStartup(message: string): void {
  const line = `[${new Date().toISOString()}] ${message}\n`;
  try {
    mkdirSync(startupLogDirectory, { recursive: true });
    appendFileSync(startupLogPath, line, 'utf8');
  } catch {
    // Logging must never prevent application startup.
  }
  console.error(line.trim());
}

if (process.platform === 'darwin') {
  app.disableHardwareAcceleration();
  app.commandLine.appendSwitch('disable-gpu');
}

process.on('uncaughtException', (error) => {
  logStartup(`uncaughtException: ${error.stack ?? error.message}`);
});

process.on('unhandledRejection', (reason) => {
  logStartup(`unhandledRejection: ${reason instanceof Error ? reason.stack ?? reason.message : String(reason)}`);
});

let mainWindow: BrowserWindow | null = null;

function analyzerBinaryPath(): string {
  const binaryName = process.platform === 'win32' ? 'akron-analyzer.exe' : 'akron-analyzer';
  const packagedPath = join(process.resourcesPath, 'akron-runtime', binaryName);
  if (existsSync(packagedPath)) {
    return packagedPath;
  }

  return join(app.getAppPath(), '..', 'target', 'release', binaryName);
}

function showStartupError(title: string, detail: string): void {
  logStartup(`${title}: ${detail}`);
  if (app.isReady()) {
    dialog.showErrorBox(title, detail);
  }
}

function createWindow(): void {
  logStartup('Creating main BrowserWindow.');
  mainWindow = new BrowserWindow({
    width: 1180,
    height: 760,
    minWidth: 900,
    minHeight: 600,
    show: true,
    backgroundColor: '#0b0d12',
    webPreferences: {
      preload: join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });

  mainWindow.webContents.on('did-fail-load', (_event, errorCode, errorDescription, validatedURL) => {
    showStartupError(
      'Akron could not load its interface',
      `Electron failed to load:\n\n${validatedURL}\n\n${errorCode}: ${errorDescription}`,
    );
  });

  mainWindow.webContents.on('render-process-gone', (_event, details) => {
    showStartupError(
      'Akron renderer stopped unexpectedly',
      `Reason: ${details.reason}${details.exitCode !== 0 ? `\nExit code: ${details.exitCode}` : ''}`,
    );
  });

  const indexPath = join(app.getAppPath(), 'dist-renderer', 'index.html');
  logStartup(`Loading renderer: ${indexPath}`);
  void mainWindow.loadFile(indexPath).catch((error: unknown) => {
    const detail = error instanceof Error ? error.message : String(error);
    showStartupError('Akron could not start', `Failed to load ${indexPath}:\n\n${detail}`);
  });

  mainWindow.on('closed', () => {
    logStartup('Main window closed.');
    mainWindow = null;
  });
}

ipcMain.handle('startup:prepare', async () => {
  const workspace = join(app.getPath('userData'), 'workspace');
  const analyzer = analyzerBinaryPath();

  logStartup('Startup preparation requested.');
  const stages = [
    { message: 'Checking application environment', action: () => app.isReady() },
    {
      message: 'Preparing local workspace',
      action: () => {
        mkdirSync(workspace, { recursive: true });
        return existsSync(workspace);
      },
    },
    {
      message: 'Verifying Akron Analyzer',
      action: () => existsSync(analyzer),
    },
    {
      message: 'Checking target platform',
      action: () => process.platform === 'darwin' || process.platform === 'win32',
    },
    { message: 'Finalizing local services', action: () => true },
  ];

  for (let index = 0; index < stages.length; index += 1) {
    const stage = stages[index];
    const success = stage.action();
    if (!success) {
      throw new Error(`${stage.message} failed.`);
    }

    const percent = Math.round(((index + 1) / stages.length) * 100);
    logStartup(`${stage.message}: ${percent}%`);
    mainWindow?.webContents.send('startup:progress', {
      message: stage.message,
      percent,
      complete: percent === 100,
    });
  }

  return { workspace, analyzer };
});

ipcMain.handle('dialog:pick-game-folder', async () => {
  const options: OpenDialogOptions = {
    title: 'Select a game folder',
    properties: ['openDirectory'],
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
  logStartup(`App ready. Electron ${process.versions.electron}; Chrome ${process.versions.chrome}; Node ${process.versions.node}; ${process.platform}/${process.arch}.`);
  createWindow();
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
}).catch((error: unknown) => {
  const detail = error instanceof Error ? error.stack ?? error.message : String(error);
  showStartupError('Akron could not initialize', detail);
});

app.on('window-all-closed', () => {
  logStartup('All windows closed.');
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
