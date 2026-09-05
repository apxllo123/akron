import { app, BrowserWindow, dialog, ipcMain, type OpenDialogOptions } from 'electron';
import { appendFileSync, existsSync, mkdirSync } from 'node:fs';
import { spawn } from 'node:child_process';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

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

function startupSplashHtml(): string {
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Akron</title>
<style>
  *{box-sizing:border-box}
  html,body{margin:0;width:100%;height:100%;overflow:hidden}
  body{font-family:-apple-system,BlinkMacSystemFont,"SF Pro Display","SF Pro Text",sans-serif;background:#090b10;color:#f4f6fb;display:grid;place-items:center}
  .wrap{width:min(520px,calc(100% - 64px));text-align:center}
  .mark{width:72px;height:72px;margin:0 auto 22px;border-radius:20px;display:grid;place-items:center;background:linear-gradient(135deg,#fff,#9aa4ff);color:#090b10;font-size:30px;font-weight:900;box-shadow:0 16px 50px rgba(0,0,0,.28)}
  .name{font-size:28px;font-weight:850;letter-spacing:.18em}
  .subtitle{margin-top:7px;color:#8e96a9;font-size:13px}
  .percent{margin-top:40px;font-size:40px;font-weight:800;letter-spacing:-.04em}
  .message{margin-top:8px;color:#aeb5c4;font-size:14px}
  .track{height:7px;margin-top:22px;border-radius:999px;background:rgba(255,255,255,.08);overflow:hidden}
  .bar{height:100%;width:0;border-radius:inherit;background:#f4f6fb;transition:width 180ms ease}
  .hint{margin-top:17px;color:#697286;font-size:12px}
  .error{margin-top:20px;color:#fca5a5;line-height:1.5;font-size:13px;display:none}
</style>
</head>
<body>
  <main class="wrap">
    <div class="mark">A</div>
    <div class="name">AKRON</div>
    <div class="subtitle">Preparing your game conversion environment</div>
    <div id="percent" class="percent">0%</div>
    <div id="message" class="message">Starting Akron…</div>
    <div class="track"><div id="bar" class="bar"></div></div>
    <div id="hint" class="hint">Initializing local services</div>
    <div id="error" class="error"></div>
  </main>
<script>
(() => {
  const percent = document.getElementById('percent');
  const message = document.getElementById('message');
  const hint = document.getElementById('hint');
  const bar = document.getElementById('bar');
  const error = document.getElementById('error');
  const mainAppUrl = ${JSON.stringify(pathToFileURL(join(__dirname, '..', 'dist-renderer', 'index.html')).toString())};

  const render = (progress) => {
    const value = Math.max(0, Math.min(100, progress.percent));
    percent.textContent = value + '%';
    message.textContent = progress.message;
    hint.textContent = progress.complete ? 'Ready' : 'Initializing local services';
    bar.style.width = value + '%';
  };

  const start = async () => {
    const unsubscribe = window.akron.onStartupProgress(render);
    try {
      render({ message: 'Starting Akron…', percent: 0, complete: false });
      await window.akron.prepareStartup();
      render({ message: 'Akron is ready', percent: 100, complete: true });
      await new Promise((resolve) => setTimeout(resolve, 140));
      window.location.replace(mainAppUrl);
    } catch (cause) {
      const text = cause instanceof Error ? cause.message : String(cause);
      error.textContent = text;
      error.style.display = 'block';
      hint.textContent = 'Startup failed';
      unsubscribe();
    }
  };

  void start();
})();
</script>
</body>
</html>`;
}

function createWindow(): void {
  logStartup('Creating main BrowserWindow.');
  mainWindow = new BrowserWindow({
    width: 1180,
    height: 760,
    minWidth: 900,
    minHeight: 600,
    show: true,
    backgroundColor: '#090b10',
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

  const splashUrl = `data:text/html;charset=UTF-8,${encodeURIComponent(startupSplashHtml())}`;
  logStartup('Loading immediate startup splash.');
  void mainWindow.loadURL(splashUrl).catch((error: unknown) => {
    const detail = error instanceof Error ? error.message : String(error);
    showStartupError('Akron could not start', `Failed to load startup splash:\n\n${detail}`);
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
    if (!stage) {
      throw new Error(`Startup stage ${index + 1} is unavailable.`);
    }

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
