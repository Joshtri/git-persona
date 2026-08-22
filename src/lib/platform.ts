import type { CapabilityStatus, PlatformCapabilities, PlatformOs } from "@/ipc/types.gen";

/**
 * Platform-aware credential copy. All user-facing wording about where secrets
 * live is derived here from the backend-reported OS, so no component hardcodes
 * Windows-specific phrasing. The labels only describe the secure *storage*
 * location — never Git HTTPS integration, which is a separate capability.
 */

/** A noun phrase naming the OS-native secure store, for inline use. */
export function secureStorageLabel(os: PlatformOs): string {
  switch (os) {
    case "Windows":
      return "Windows Credential Manager";
    case "Macos":
      return "your system Keychain";
    default:
      return "your system credential vault";
  }
}

/** A full sentence stating where credentials are stored on this platform. */
export function secureStorageSentence(os: PlatformOs): string {
  return `Credentials are securely stored in ${secureStorageLabel(os)}.`;
}

export function isAvailable(status: CapabilityStatus): boolean {
  return status === "Available";
}

/** Human-readable OS family name for build-info style displays. */
export function osDisplayName(os: PlatformOs): string {
  switch (os) {
    case "Windows":
      return "Windows";
    case "Macos":
      return "macOS";
    case "Linux":
      return "Linux";
    default:
      return "Unknown";
  }
}

/**
 * Empty-state description for the credentials list. Only promises automatic Git
 * hand-off where it is actually implemented (Windows today); elsewhere it
 * describes secure storage without over-claiming.
 */
export function credentialsEmptyDescription(caps: PlatformCapabilities): string {
  if (isAvailable(caps.git_credential_integration)) {
    return "Add an HTTPS token so applying a profile can switch your Git credential automatically.";
  }
  return `Add an HTTPS token to store it securely in ${secureStorageLabel(caps.os)} and switch it with a profile inside GitPersona.`;
}
