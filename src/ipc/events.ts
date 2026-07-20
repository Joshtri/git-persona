import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ScanProgress, SmartSwitchStatus, SwitchEvent } from "./types.gen";

export function onSettingsChanged(cb: (payload: unknown) => void): Promise<UnlistenFn> {
  return listen("settings-changed", (e) => cb(e.payload));
}

export function onProfilesChanged(cb: (payload: unknown) => void): Promise<UnlistenFn> {
  return listen("profiles-changed", (e) => cb(e.payload));
}

export function onRepoScanProgress(cb: (progress: ScanProgress) => void): Promise<UnlistenFn> {
  return listen<ScanProgress>("repo-scan-progress", (e) => cb(e.payload));
}

export function onSmartSwitch(cb: (event: SwitchEvent) => void): Promise<UnlistenFn> {
  return listen<SwitchEvent>("smart-switch", (e) => cb(e.payload));
}

export function onSmartSwitchStatus(cb: (status: SmartSwitchStatus) => void): Promise<UnlistenFn> {
  return listen<SmartSwitchStatus>("smart-switch-status", (e) => cb(e.payload));
}
