pub(crate) mod audit_jsonl;
pub(crate) mod credential_store_tauri;
#[cfg(not(windows))]
pub(crate) mod credential_vault_noop;
#[cfg(windows)]
pub(crate) mod credential_vault_windows;
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
