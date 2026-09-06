declare global {
  interface Window {
    akron: {
      getAppInfo(): Promise<{
        name: 'Akron';
        version: string;
        platform: 'darwin' | 'win32';
        arch: string;
      }>;
      prepareStartup(): Promise<{ workspace: string; analyzer: string }>;
      onStartupProgress(listener: (progress: {
        message: string;
        percent: number;
        complete: boolean;
      }) => void): () => void;
      pickGameFolder(): Promise<string | null>;
      analyzeGame(gamePath: string): Promise<AnalysisReport>;
      buildAdaptationPlan(profile: GameProfile): Promise<AdaptationPlan>;
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
  protection?: {
    packers_or_protectors: string[];
    anti_cheats: string[];
  };
}

interface GameProfile {
  executables: Array<{
    path: string;
    architecture?: string;
    format: string;
  }>;
  graphics: {
    direct3d9: boolean;
    direct3d10: boolean;
    direct3d11: boolean;
    direct3d12: boolean;
    dxgi: boolean;
    vulkan: boolean;
    opengl: boolean;
  };
  windows_apis: Array<{ family: string; evidence: string[] }>;
  runtimes: Array<{ name: string; evidence: string[] }>;
  protections: {
    packers_or_protectors: string[];
    anti_cheats: string[];
  };
}

interface GameManifest {
  files: GameManifestFile[];
  executables: GameManifestExecutable[];
}

interface AnalysisReport extends GameManifest {
  profile: GameProfile;
}

interface AdaptationPlan {
  steps: AdaptationStep[];
  required_modules: string[];
}

interface AdaptationStep {
  id: string;
  title: string;
  description: string;
  module: string;
  status: 'planned' | 'ready' | 'blocked';
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
const graphicsValue = document.querySelector<HTMLElement>('#graphics-value');
const apiValue = document.querySelector<HTMLElement>('#api-value');
const runtimeValue = document.querySelector<HTMLElement>('#runtime-value');
const protectionValue = document.querySelector<HTMLElement>('#protection-value');
const planButton = document.querySelector<HTMLButtonElement>('#plan-conversion');
const planCard = document.querySelector<HTMLElement>('#plan-card');
const planSteps = document.querySelector<HTMLElement>('#plan-steps');
const planCount = document.querySelector<HTMLElement>('#plan-count');

let latestProfile: GameProfile | null = null;

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
  if (planButton) planButton.disabled = busy || !latestProfile;
}

function showError(message: string | null): void {
  if (!error) return;
  error.textContent = message ?? '';
  error.hidden = message === null;
}

function renderTags(container: HTMLElement | null, values: string[], emptyLabel: string): void {
  if (!container) return;
  container.replaceChildren();
  if (values.length === 0) {
    const empty = document.createElement('span');
    empty.className = 'muted';
    empty.textContent = emptyLabel;
    container.append(empty);
    return;
  }
  for (const value of values) {
    const tag = document.createElement('span');
    tag.className = 'tag';
    tag.textContent = value;
    container.append(tag);
  }
}

function renderProfile(profile: GameProfile): void {
  const graphics: string[] = [];
  if (profile.graphics.direct3d9) graphics.push('Direct3D 9');
  if (profile.graphics.direct3d10) graphics.push('Direct3D 10');
  if (profile.graphics.direct3d11) graphics.push('Direct3D 11');
  if (profile.graphics.direct3d12) graphics.push('Direct3D 12');
  if (profile.graphics.dxgi) graphics.push('DXGI');
  if (profile.graphics.vulkan) graphics.push('Vulkan');
  if (profile.graphics.opengl) graphics.push('OpenGL');

  renderTags(graphicsValue, graphics, 'None detected');
  renderTags(apiValue, profile.windows_apis.map((item) => item.family), 'None detected');
  renderTags(runtimeValue, profile.runtimes.map((item) => item.name), 'None detected');
  renderTags(
    protectionValue,
    [...profile.protections.packers_or_protectors, ...profile.protections.anti_cheats],
    'No protection signals',
  );
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

function renderPlan(plan: AdaptationPlan): void {
  if (!planCard || !planSteps || !planCount) return;
  planSteps.replaceChildren();
  planCount.textContent = `${plan.steps.length} ${plan.steps.length === 1 ? 'step' : 'steps'}`;

  for (const [index, step] of plan.steps.entries()) {
    const item = document.createElement('article');
    item.className = `plan-step ${step.status}`;

    const number = document.createElement('div');
    number.className = 'plan-step-number';
    number.textContent = String(index + 1).padStart(2, '0');

    const body = document.createElement('div');
    body.className = 'plan-step-body';

    const title = document.createElement('h3');
    title.textContent = step.title;
    const description = document.createElement('p');
    description.textContent = step.description;
    const module = document.createElement('span');
    module.className = 'plan-module';
    module.textContent = step.module;

    body.append(title, description, module);

    const badge = document.createElement('span');
    badge.className = 'plan-status';
    badge.textContent = step.status === 'ready' ? 'Ready' : step.status === 'blocked' ? 'Blocked' : 'Planned';

    item.append(number, body, badge);
    planSteps.append(item);
  }

  planCard.hidden = false;
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
  planCard?.setAttribute('hidden', '');
  latestProfile = null;
  const path = await window.akron.pickGameFolder();
  if (!path) return;

  if (selectedPath) selectedPath.textContent = path;
  setStatus('Analyzing game…', true);
  if (results) results.hidden = true;

  try {
    const report = await window.akron.analyzeGame(path);
    renderManifest(report);
    latestProfile = report.profile;
    renderProfile(report.profile);
    setStatus('Analysis complete', false);
  } catch (cause) {
    const message = cause instanceof Error ? cause.message : String(cause);
    showError(message);
    setStatus('Analysis failed', false);
  }
});

planButton?.addEventListener('click', async () => {
  if (!latestProfile) return;
  showError(null);
  setStatus('Building conversion plan…', true);
  planButton.disabled = true;
  try {
    const plan = await window.akron.buildAdaptationPlan(latestProfile);
    renderPlan(plan);
    setStatus('Conversion plan ready', false);
  } catch (cause) {
    const message = cause instanceof Error ? cause.message : String(cause);
    showError(message);
    setStatus('Plan generation failed', false);
  } finally {
    planButton.disabled = false;
  }
});

void initializeAkron();

export {};
