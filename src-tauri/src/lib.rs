mod commands;
mod domain;
mod error;
mod infra;
mod logging;
mod services;
mod state;

use tauri::Manager;

use commands::{
    activity::activity_list,
    credentials::{
        credential_assign_profile, credential_create, credential_delete, credential_list,
        credential_open_manager, credential_switch, credential_update,
    },
    profiles::{
        profile_apply, profile_create, profile_delete, profile_get_active, profile_list,
        profile_update,
    },
    repos::{
        repo_assign_profile, repo_get, repo_list, repo_refresh, repo_remove, repo_reveal,
        repo_scan, repo_toggle_favorite,
    },
    settings::{settings_get, settings_set},
    ssh::{
        ssh_assign_profile, ssh_config_open, ssh_config_preview, ssh_generate, ssh_import,
        ssh_list, ssh_remove, ssh_reveal, ssh_reveal_folder, ssh_scan,
    },
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[allow(clippy::expect_used, clippy::missing_panics_doc)]
pub fn run() {
    logging::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state =
                state::AppState::init(app.handle()).expect("fatal: failed to initialise AppState");
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            settings_get,
            settings_set,
            profile_list,
            profile_create,
            profile_update,
            profile_delete,
            profile_apply,
            profile_get_active,
            activity_list,
            repo_scan,
            repo_list,
            repo_get,
            repo_refresh,
            repo_remove,
            repo_assign_profile,
            repo_toggle_favorite,
            repo_reveal,
            ssh_list,
            ssh_scan,
            ssh_import,
            ssh_generate,
            ssh_remove,
            ssh_assign_profile,
            ssh_reveal,
            ssh_reveal_folder,
            ssh_config_open,
            ssh_config_preview,
            credential_list,
            credential_create,
            credential_update,
            credential_assign_profile,
            credential_switch,
            credential_delete,
            credential_open_manager,
        ])
        .run(tauri::generate_context!())
        .expect("fatal: error while running tauri application");
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod export_bindings {
    use crate::domain::{
        audit::AuditEntry,
        credential::{Credential, Protocol},
        identity::Identity,
        profile::Profile,
        repo::{Repo, ScanProgress},
        settings::{AppSettings, Theme},
        ssh::{SshAlgorithm, SshKey},
    };
    use ts_rs::TS;

    #[test]
    fn export_ts_bindings() {
        Identity::export_all_to("../src/ipc/").unwrap();
        Profile::export_all_to("../src/ipc/").unwrap();
        Repo::export_all_to("../src/ipc/").unwrap();
        ScanProgress::export_all_to("../src/ipc/").unwrap();
        AuditEntry::export_all_to("../src/ipc/").unwrap();
        AppSettings::export_all_to("../src/ipc/").unwrap();
        Theme::export_all_to("../src/ipc/").unwrap();
        SshKey::export_all_to("../src/ipc/").unwrap();
        SshAlgorithm::export_all_to("../src/ipc/").unwrap();
        Credential::export_all_to("../src/ipc/").unwrap();
        Protocol::export_all_to("../src/ipc/").unwrap();
    }
}
