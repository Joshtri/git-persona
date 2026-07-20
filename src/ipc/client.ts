import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  AuditEntry,
  Credential,
  Profile,
  Repo,
  SshAlgorithm,
  SshKey,
} from "./types.gen";

export async function settingsGet(): Promise<AppSettings> {
  return invoke<AppSettings>("settings_get");
}

export async function settingsSet(settings: AppSettings): Promise<AppSettings> {
  return invoke<AppSettings>("settings_set", { settings });
}

export async function profileList(): Promise<Profile[]> {
  return invoke<Profile[]>("profile_list");
}

export async function profileCreate(
  label: string,
  name: string,
  email: string,
  signing_key?: string | null,
  color?: string | null
): Promise<Profile> {
  return invoke<Profile>("profile_create", {
    label,
    name,
    email,
    signingKey: signing_key,
    color,
  });
}

export async function profileUpdate(
  id: string,
  label: string,
  name: string,
  email: string,
  signing_key?: string | null,
  color?: string | null
): Promise<Profile> {
  return invoke<Profile>("profile_update", {
    id,
    label,
    name,
    email,
    signingKey: signing_key,
    color,
  });
}

export async function profileDelete(id: string): Promise<void> {
  return invoke<void>("profile_delete", { id });
}

export async function profileApply(id: string): Promise<void> {
  return invoke<void>("profile_apply", { id });
}

export async function profileGetActive(): Promise<Profile | null> {
  return invoke<Profile | null>("profile_get_active");
}

export async function activityList(): Promise<AuditEntry[]> {
  return invoke<AuditEntry[]>("activity_list");
}

export async function repoScan(paths: string[]): Promise<Repo[]> {
  return invoke<Repo[]>("repo_scan", { paths });
}

export async function repoList(): Promise<Repo[]> {
  return invoke<Repo[]>("repo_list");
}

export async function repoGet(id: string): Promise<Repo> {
  return invoke<Repo>("repo_get", { id });
}

export async function repoRefresh(id: string): Promise<Repo> {
  return invoke<Repo>("repo_refresh", { id });
}

export async function repoRemove(id: string): Promise<void> {
  return invoke<void>("repo_remove", { id });
}

export async function repoAssignProfile(id: string, profileId: string | null): Promise<Repo> {
  return invoke<Repo>("repo_assign_profile", { id, profileId });
}

export async function repoToggleFavorite(id: string): Promise<Repo> {
  return invoke<Repo>("repo_toggle_favorite", { id });
}

export async function repoReveal(id: string): Promise<void> {
  return invoke<void>("repo_reveal", { id });
}

export async function sshList(): Promise<SshKey[]> {
  return invoke<SshKey[]>("ssh_list");
}

export async function sshScan(): Promise<SshKey[]> {
  return invoke<SshKey[]>("ssh_scan");
}

export async function sshImport(
  privateKeyPath: string,
  hostAlias?: string | null,
  hostName?: string | null
): Promise<SshKey> {
  return invoke<SshKey>("ssh_import", { privateKeyPath, hostAlias, hostName });
}

export async function sshGenerate(
  label: string,
  algorithm: SshAlgorithm,
  comment: string | null,
  outputDir: string,
  fileName: string,
  hostAlias?: string | null,
  hostName?: string | null
): Promise<SshKey> {
  return invoke<SshKey>("ssh_generate", {
    label,
    algorithm,
    comment,
    outputDir,
    fileName,
    hostAlias,
    hostName,
  });
}

export async function sshRemove(id: string): Promise<void> {
  return invoke<void>("ssh_remove", { id });
}

export async function sshAssignProfile(id: string, profileId: string | null): Promise<SshKey> {
  return invoke<SshKey>("ssh_assign_profile", { id, profileId });
}

export async function sshReveal(id: string): Promise<void> {
  return invoke<void>("ssh_reveal", { id });
}

export async function sshRevealFolder(): Promise<void> {
  return invoke<void>("ssh_reveal_folder");
}

export async function sshConfigOpen(): Promise<void> {
  return invoke<void>("ssh_config_open");
}

export async function sshConfigPreview(): Promise<string> {
  return invoke<string>("ssh_config_preview");
}

export async function credentialList(): Promise<Credential[]> {
  return invoke<Credential[]>("credential_list");
}

export async function credentialCreate(
  profileId: string | null,
  host: string,
  username: string,
  token: string
): Promise<Credential> {
  return invoke<Credential>("credential_create", { profileId, host, username, token });
}

export async function credentialUpdate(
  id: string,
  username: string,
  token: string | null
): Promise<Credential> {
  return invoke<Credential>("credential_update", { id, username, token });
}

export async function credentialAssignProfile(
  id: string,
  profileId: string | null
): Promise<Credential> {
  return invoke<Credential>("credential_assign_profile", { id, profileId });
}

export async function credentialSwitch(id: string): Promise<Credential> {
  return invoke<Credential>("credential_switch", { id });
}

export async function credentialDelete(id: string): Promise<void> {
  return invoke<void>("credential_delete", { id });
}

export async function credentialOpenManager(): Promise<void> {
  return invoke<void>("credential_open_manager");
}
