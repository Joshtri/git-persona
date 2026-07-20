import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ScanProgress } from "./types.gen";

export function onSettingsChanged(cb: (payload: unknown) => void): Promise<UnlistenFn> {
  return listen("settings-changed", (e) => cb(e.payload));
}

export function onProfilesChanged(cb: (payload: unknown) => void): Promise<UnlistenFn> {
  return listen("profiles-changed", (e) => cb(e.payload));
}

export function onRepoScanProgress(cb: (progress: ScanProgress) => void): Promise<UnlistenFn> {
  return listen<ScanProgress>("repo-scan-progress", (e) => cb(e.payload));
}
