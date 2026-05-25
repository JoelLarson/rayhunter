import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { openUrl } from '@tauri-apps/plugin-opener';
import { getCurrentWindow } from '@tauri-apps/api/window';

export interface DetectedDevice {
    id: string;
    subcommand: string;
    display_name: string;
    admin_ip?: string;
}

export function autodetectDevice(): Promise<DetectedDevice[]> {
    return invoke('autodetect_device');
}

export function installRayhunter(args: string[]): Promise<void> {
    return invoke('install_rayhunter', { args });
}

export { listen, openUrl, getCurrentWindow };
