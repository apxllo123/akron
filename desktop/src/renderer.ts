declare global {
  interface Window {
    akron: {
      getAppInfo(): Promise<{
        name: 'Akron';
        version: string;
        platform: 'darwin' | 'win32';
        arch: string;
      }>;
      pickGameFolder(): Promise<string | null>;
      analyzeGame(gamePath: string): Promise<unknown>;
    };
  }
}

interface GameManifestFile {
  path: string;
  size: number;
  sha256: string;
  extension?: string;
}

interface GameManifestExecutable {
  path: string;
  format: string;
  architecture?: string;
}

interface GameManifest {
  files: GameManifestFile[];
  executables: GameManifestExecutable[];
}

const selectButton = document.querySelector<HTMLButtonElement>('#select-folder');
const selectedPath = document.querySelector<HTMLElement>('#selected-path');
const status = document.querySelector<HTMLElement>('#status');
const error = document.querySelector<HTMLElement>('#error');
const results = document.querySelector<HTMLElement>('#results');
const filesValue = document.querySelector<HTMLElement>('#files-value');
const executablesValue = document.querySelector<HTMLElement>('#executables-value');
const peValue = document.querySelector<HTMLElement>('#pe-value');
const sizeValue = document.querySelector<HTMLElement>('#size-value');
const version = document.querySelector<HTMLElement>('#version');

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB', 'TB'];
  let value = bytes;
  let index = -1;
  do {
    value /= 1024;
    index += 1;
  } while (value >= 1024 && index < units.length - 1);
  return `${value.toFixed(1)} ${units[index]}`;
}

function setStatus(message: string, busy: boolean): void {
  if (status) status.textContent = message;
  if (selectButton) selectButton.disabled = busy;
}

function showError(message: string | null): void {
  if (!error) return;
  error.textContent = message ?? '';
  error.hidden = message === null;
}

function renderManifest(manifest: GameManifest): void {
  if (filesValue) filesValue.textContent = String(manifest.files.length);
  if (executablesValue) executablesValue.textContent = String(manifest.executables.length);
  if (peValue) {
    peValue.textContent = String(manifest.executables.filter((item) => item.format === 'PE').length);
  }
  if (sizeValue) {
    sizeValue.textContent = formatBytes(manifest.files.reduce((total, item) => total + item.size, 0));
  }
  if (results) results.hidden = false;
}

selectButton?.addEventListener('click', async () => {
  showError(null);
  const path = await window.akron.pickGameFolder();
  if (!path) return;

  if (selectedPath) selectedPath.textContent = path;
  setStatus('Analyzing game…', true);
  if (results) results.hidden = true;

  try {
    const manifest = (await window.akron.analyzeGame(path)) as GameManifest;
    renderManifest(manifest);
    setStatus('Analysis complete', false);
  } catch (cause) {
    const message = cause instanceof Error ? cause.message : String(cause);
    showError(message);
    setStatus('Analysis failed', false);
  }
});

void window.akron.getAppInfo().then((info) => {
  if (version) version.textContent = `v${info.version} · ${info.arch}`;
});

export {};
