import { vi } from "vitest";
import type { AppSettings } from "@/ipc/types.gen";

export const mockSettings: AppSettings = {
  theme: "Dark",
  show_audit_log: true,
  auto_scan_repos: false,
};

vi.mock("@/ipc/client", () => ({
  settingsGet: vi.fn().mockResolvedValue(mockSettings),
  settingsSet: vi.fn().mockImplementation(async (s: AppSettings) => s),
  profileList: vi.fn().mockResolvedValue([]),
  profileCreate: vi.fn().mockRejectedValue({ code: "NOT_IMPLEMENTED", message: "not implemented" }),
  profileDelete: vi.fn().mockRejectedValue({ code: "NOT_IMPLEMENTED", message: "not implemented" }),
  repoScan: vi.fn().mockRejectedValue({ code: "NOT_IMPLEMENTED", message: "not implemented" }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));
