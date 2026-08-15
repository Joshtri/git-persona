import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { openUrl as openerOpenUrl } from "@tauri-apps/plugin-opener";
import type {
  AppSettings,
  AuditEntry,
  BootstrapData,
  Credential,
  Profile,
  Repo,
  RepoGroup,
  Rule,
  RuleOperator,
  RulePreviewInput,
  RulePreviewResult,
  RuleSubject,
  RuleSummary,
  SmartSwitchStatus,
  SshAlgorithm,
  SshKey,
  SystemScan,
} from "./types.gen";

export async function appVersion(): Promise<string> {
  return getVersion();
}

export async function openUrl(url: string): Promise<void> {
  return openerOpenUrl(url);
}

export async function bootstrapFetch(): Promise<BootstrapData> {
  return invoke<BootstrapData>("bootstrap_fetch");
}

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

export async function repoGroupList(): Promise<RepoGroup[]> {
  return invoke<RepoGroup[]>("repo_group_list");
}

export async function repoGroupCreate(name: string, color: string | null): Promise<RepoGroup> {
  return invoke<RepoGroup>("repo_group_create", { name, color });
}

export async function repoGroupUpdate(
  id: string,
  name: string,
  color: string | null
): Promise<RepoGroup> {
  return invoke<RepoGroup>("repo_group_update", { id, name, color });
}

export async function repoGroupDelete(id: string): Promise<void> {
  return invoke<void>("repo_group_delete", { id });
}

export async function repoSetGroup(id: string, groupId: string | null): Promise<Repo> {
  return invoke<Repo>("repo_set_group", { id, groupId });
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
  token: string,
  pin: string | null
): Promise<Credential> {
  return invoke<Credential>("credential_create", { profileId, host, username, token, pin });
}

export async function credentialSetPin(id: string, pin: string): Promise<Credential> {
  return invoke<Credential>("credential_set_pin", { id, pin });
}

export async function credentialReveal(id: string, pin: string): Promise<string> {
  return invoke<string>("credential_reveal", { id, pin });
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

export async function ruleList(): Promise<Rule[]> {
  return invoke<Rule[]>("rule_list");
}

export async function ruleGet(id: string): Promise<Rule> {
  return invoke<Rule>("rule_get", { id });
}

export async function ruleCreate(
  name: string,
  subject: RuleSubject,
  operator: RuleOperator,
  value: string,
  targetProfileId: string
): Promise<Rule> {
  return invoke<Rule>("rule_create", { name, subject, operator, value, targetProfileId });
}

export async function ruleUpdate(
  id: string,
  name: string,
  subject: RuleSubject,
  operator: RuleOperator,
  value: string,
  targetProfileId: string
): Promise<Rule> {
  return invoke<Rule>("rule_update", { id, name, subject, operator, value, targetProfileId });
}

export async function ruleDelete(id: string): Promise<void> {
  return invoke<void>("rule_delete", { id });
}

export async function ruleDuplicate(id: string): Promise<Rule> {
  return invoke<Rule>("rule_duplicate", { id });
}

export async function ruleSetEnabled(id: string, enabled: boolean): Promise<Rule> {
  return invoke<Rule>("rule_set_enabled", { id, enabled });
}

export async function ruleSetAllEnabled(enabled: boolean): Promise<Rule[]> {
  return invoke<Rule[]>("rule_set_all_enabled", { enabled });
}

export async function ruleReorder(orderedIds: string[]): Promise<Rule[]> {
  return invoke<Rule[]>("rule_reorder", { orderedIds });
}

export async function rulePreview(input: RulePreviewInput): Promise<RulePreviewResult> {
  return invoke<RulePreviewResult>("rule_preview", { input });
}

export async function ruleSummary(): Promise<RuleSummary> {
  return invoke<RuleSummary>("rule_summary");
}

export async function ruleExport(path: string): Promise<void> {
  return invoke<void>("rule_export", { path });
}

export async function ruleImport(path: string, replace: boolean): Promise<Rule[]> {
  return invoke<Rule[]>("rule_import", { path, replace });
}

export async function smartSwitchStatus(): Promise<SmartSwitchStatus> {
  return invoke<SmartSwitchStatus>("smart_switch_status");
}

export async function smartSwitchSetEnabled(enabled: boolean): Promise<SmartSwitchStatus> {
  return invoke<SmartSwitchStatus>("smart_switch_set_enabled", { enabled });
}

export async function smartSwitchRestart(): Promise<SmartSwitchStatus> {
  return invoke<SmartSwitchStatus>("smart_switch_restart");
}

export async function smartSwitchPause(): Promise<SmartSwitchStatus> {
  return invoke<SmartSwitchStatus>("smart_switch_pause");
}

export async function smartSwitchResume(): Promise<SmartSwitchStatus> {
  return invoke<SmartSwitchStatus>("smart_switch_resume");
}

export async function smartSwitchConfirm(gitRoot: string): Promise<void> {
  return invoke<void>("smart_switch_confirm", { gitRoot });
}

export async function smartSwitchCancel(gitRoot: string): Promise<void> {
  return invoke<void>("smart_switch_cancel", { gitRoot });
}

export async function onboardingScan(): Promise<SystemScan> {
  return invoke<SystemScan>("onboarding_scan");
}

export async function onboardingApply(
  createProfile: boolean,
  label: string,
  name: string,
  email: string,
  signingKey: string | null,
  color: string | null,
  sshPaths: string[]
): Promise<void> {
  return invoke<void>("onboarding_apply", {
    createProfile,
    label,
    name,
    email,
    signingKey,
    color,
    sshPaths,
  });
}

export async function onboardingSkip(): Promise<void> {
  return invoke<void>("onboarding_skip");
}
