declare global {
  interface Window {
    akron: {
      getAppInfo(): Promise<{
        name: 'Akron';
        version: string;
        platform: 'darwin' | 'win32';
        arch: string;
      }>;
      prepareStartup(): Promise<{
        workspace: string;
        analyzer: string;
      }>;
      onStartupProgress(listener: (progress: {
        message: string;
        percent: number;
        complete: boolean;
      }) => void): () => void;
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

const startup = document.querySelector<HTMLElement>('#startup');
const startupProgress = document.querySelector<HTMLElement>('#startup-progress');
const startupPercent = document.querySelector<HTMLElement>('#startup-percent');
const startupMessage = document.querySelector<HTMLElement>('#startup-message');
const startupDetail = document.querySelector<HTMLElement>('#startup-detail');
const startupStages = document.querySelector<HTMLElement>('#startup-stages');
const appShell = document.querySelector<HTMLElement>('#app');

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

function updateStartup(progress: { message: string; percent: number; complete: boolean }): void {
  const percent = Math.max(0, Math.min(100, progress.percent));
  if (startupProgress) startupProgress.style.width = `${percent}%`;
  if (startupPercent) startupPercent.textContent = `${percent}%`;
  if (startupMessage) startupMessage.textContent = progress.message;
  if (startupDetail) startupDetail.textContent = progress.complete ? 'Ready' : 'Initializing';

  if (startupStages && !progress.complete) {
    const existing = Array.from(startupStages.querySelectorAll<HTMLElement>('[data-startup-message]'));
    if (!existing.some((item) => item.dataset.startupMessage === progress.message)) {
      const stage = document.createElement('div');
      stage.className = 'startup-stage complete';
      stage.dataset.startupMessage = progress.message;
      stage.innerHTML = `<span class="startup-stage-icon">✓</span><span>${progress.message}</span>`;
      startupStages.append(stage);
    }
  }
}

async function initializeAkron(): Promise<void> {
  updateStartup({ message: 'Starting Akron…', percent: 0, complete: false });
  const unsubscribe = window.akron.onStartupProgress(updateStartup);

  try {
    await window.akron.getAppInfo().then((info) => {
      if (version) version.textContent = `v${info.version} · ${info.arch}`;
    });

    await window.akron.prepareStartup();
    updateStartup({ message: 'Akron is ready', percent: 100, complete: true });

    window.setTimeout(() => {
      if (startup) startup.hidden = true;
      if (appShell) appShell.hidden = false;
    }, 250);
  } catch (cause) {
    const message = cause instanceof Error ? cause.message : String(cause);
    updateStartup({ message: 'Startup failed', percent: 100, complete: false });
    if (startupDetail) startupDetail.textContent = message;

    const stage = document.createElement('div');
    stage.className = 'startup-stage failed';
    stage.innerHTML = `<span class="startup-stage-icon">!</span><span>${message}</span>`;
    startupStages?.append(stage);
  } finally {
    unsubscribe();
  }
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

void initializeAkron();

export {};
