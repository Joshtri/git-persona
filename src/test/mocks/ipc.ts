import { vi } from "vitest";
import type { AppSettings } from "@/ipc/types.gen";

export const mockSettings: AppSettings = {
  theme: "Dark",
  show_audit_log: true,
  auto_scan_repos: false,
  smart_switching: {
    enabled: false,
    auto_ssh: true,
    auto_credential: true,
    show_notification: true,
    confirm_before_switch: false,
    start_on_launch: false,
  },
};

const mockSmartStatus = {
  enabled: false,
  watching: false,
  paused: false,
  repos_monitored: 0,
  last_switch: null,
  started_at: null,
};

vi.mock("@/ipc/client", () => ({
  settingsGet: vi.fn().mockResolvedValue(mockSettings),
  settingsSet: vi.fn().mockImplementation(async (s: AppSettings) => s),
  profileList: vi.fn().mockResolvedValue([]),
  profileCreate: vi.fn().mockRejectedValue({ code: "NOT_IMPLEMENTED", message: "not implemented" }),
  profileDelete: vi.fn().mockRejectedValue({ code: "NOT_IMPLEMENTED", message: "not implemented" }),
  repoScan: vi.fn().mockRejectedValue({ code: "NOT_IMPLEMENTED", message: "not implemented" }),
  smartSwitchStatus: vi.fn().mockResolvedValue(mockSmartStatus),
  smartSwitchSetEnabled: vi.fn().mockResolvedValue(mockSmartStatus),
  smartSwitchRestart: vi.fn().mockResolvedValue(mockSmartStatus),
  smartSwitchPause: vi.fn().mockResolvedValue(mockSmartStatus),
  smartSwitchResume: vi.fn().mockResolvedValue(mockSmartStatus),
  smartSwitchConfirm: vi.fn().mockResolvedValue(undefined),
  smartSwitchCancel: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));
