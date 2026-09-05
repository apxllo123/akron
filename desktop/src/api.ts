import { app, ipcMain } from 'electron';

export type PlatformName = 'darwin' | 'win32';

export interface AkronAppInfo {
  name: 'Akron';
  version: string;
  platform: PlatformName;
  arch: string;
}

export function registerApiHandlers(): void {
  ipcMain.handle('app:info', (): AkronAppInfo => ({
    name: 'Akron',
    version: app.getVersion(),
    platform: process.platform as PlatformName,
    arch: process.arch,
  }));
}
