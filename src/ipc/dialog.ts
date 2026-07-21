import { open, save } from "@tauri-apps/plugin-dialog";

/**
 * Prompt the user to pick one or more folders to scan for Git repositories.
 * Returns an empty array when the dialog is cancelled.
 */
export async function pickFolders(): Promise<string[]> {
  const selected = await open({ directory: true, multiple: true });
  if (selected == null) return [];
  return Array.isArray(selected) ? selected : [selected];
}

/**
 * Prompt the user to pick a single private key file to import.
 * Returns null when the dialog is cancelled.
 */
export async function pickPrivateKeyFile(): Promise<string | null> {
  const selected = await open({ directory: false, multiple: false });
  return typeof selected === "string" ? selected : null;
}

/**
 * Prompt the user to pick a single output directory (e.g. for key generation).
 * Returns null when the dialog is cancelled.
 */
export async function pickDirectory(): Promise<string | null> {
  const selected = await open({ directory: true, multiple: false });
  return typeof selected === "string" ? selected : null;
}

const RULES_FILTER = [{ name: "Rules", extensions: ["json"] }];

/**
 * Prompt for a destination path to export the rules file to.
 * Returns null when the dialog is cancelled.
 */
export async function pickRulesExportPath(): Promise<string | null> {
  return save({ defaultPath: "rules.json", filters: RULES_FILTER });
}

/**
 * Prompt for a rules JSON file to import.
 * Returns null when the dialog is cancelled.
 */
export async function pickRulesImportFile(): Promise<string | null> {
  const selected = await open({
    directory: false,
    multiple: false,
    filters: RULES_FILTER,
  });
  return typeof selected === "string" ? selected : null;
}
