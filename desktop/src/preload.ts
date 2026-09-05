import { contextBridge, ipcRenderer } from 'electron';

export interface AkronApi {
  getAppInfo(): Promise<{
    name: 'Akron';
    version: string;
    platform: 'darwin' | 'win32';
    arch: string;
  }>;
  pickGameFolder(): Promise<string | null>;
  analyzeGame(gamePath: string): Promise<unknown>;
}

const api: AkronApi = {
  getAppInfo: () => ipcRenderer.invoke('app:info'),
  pickGameFolder: () => ipcRenderer.invoke('dialog:pick-game-folder'),
  analyzeGame: (gamePath) => ipcRenderer.invoke('analyzer:analyze', gamePath),
};

contextBridge.exposeInMainWorld('akron', api);
