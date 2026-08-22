pub(crate) mod audit_jsonl;
pub(crate) mod commit_hook_fs;
pub(crate) mod credential_store_tauri;
/// macOS Keychain / Linux Secret Service via the `keyring` crate.
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) mod credential_vault_keyring;
/// Fallback for platforms without a secure OS vault integration.
#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub(crate) mod credential_vault_noop;
#[cfg(windows)]
pub(crate) mod credential_vault_windows;
#[cfg(windows)]
pub(crate) mod shell_windows;
pub(crate) mod git_config_gix;
pub(crate) mod git_dir_watcher;
pub(crate) mod git_meta;
pub(crate) mod paths;
pub(crate) mod profile_store_tauri;
pub(crate) mod repo_scanner_fs;
pub(crate) mod repo_store_tauri;
pub(crate) mod rule_store_tauri;
pub(crate) mod ssh_config_writer;
pub(crate) mod ssh_key_generator;
pub(crate) mod ssh_key_reader;
pub(crate) mod ssh_scanner_fs;
pub(crate) mod ssh_store_tauri;
pub(crate) mod switch_observer_tauri;
