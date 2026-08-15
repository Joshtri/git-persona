mod api;
mod commands;
mod domain;
mod error;
mod infra;
mod logging;
mod services;
mod state;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

use commands::{
    activity::activity_list,
    bootstrap::bootstrap_fetch,
    credentials::{
        credential_assign_profile, credential_create, credential_delete, credential_list,
        credential_open_manager, credential_reveal, credential_set_pin, credential_switch,
        credential_update,
    },
    identity_switch::{
        smart_switch_cancel, smart_switch_confirm, smart_switch_pause, smart_switch_restart,
        smart_switch_resume, smart_switch_set_enabled, smart_switch_status,
    },
    onboarding::{onboarding_apply, onboarding_scan, onboarding_skip},
    profiles::{
        profile_apply, profile_create, profile_delete, profile_get_active, profile_list,
        profile_update,
    },
    repos::{
        repo_assign_profile, repo_get, repo_group_create, repo_group_delete, repo_group_list,
        repo_group_update, repo_list, repo_refresh, repo_remove, repo_reveal, repo_scan,
        repo_set_group, repo_toggle_favorite,
    },
    rules::{
        rule_create, rule_delete, rule_duplicate, rule_export, rule_get, rule_import, rule_list,
        rule_preview, rule_reorder, rule_set_all_enabled, rule_set_enabled, rule_summary,
        rule_update,
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
    // Load .env at the project root for local dev overrides (e.g. GITPERSONA_SERVER_URL).
    // Silently ignored when no .env file exists (production installs).
    let _ = dotenvy::dotenv();
    logging::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        // Autostart entry launches the app with `--hidden` so a login start comes
        // up straight into the tray without flashing the window.
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .setup(setup_app)
        .on_window_event(|window, event| {
            // Close-to-tray: intercept the window close and hide instead of
            // quitting, so the watcher keeps running in the background. When the
            // preference is off, let the default close (and app exit) proceed.
            if let WindowEvent::CloseRequested { api, .. } = event {
                let keep_running = window
                    .try_state::<state::AppState>()
                    .and_then(|s| s.settings.get().ok())
                    .is_some_and(|s| s.close_to_tray);
                if keep_running {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap_fetch,
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
            repo_group_list,
            repo_group_create,
            repo_group_update,
            repo_group_delete,
            repo_set_group,
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
            credential_reveal,
            credential_set_pin,
            credential_open_manager,
            rule_list,
            rule_get,
            rule_create,
            rule_update,
            rule_delete,
            rule_duplicate,
            rule_set_enabled,
            rule_set_all_enabled,
            rule_reorder,
            rule_preview,
            rule_summary,
            rule_export,
            rule_import,
            smart_switch_status,
            smart_switch_set_enabled,
            smart_switch_restart,
            smart_switch_pause,
            smart_switch_resume,
            smart_switch_confirm,
            smart_switch_cancel,
            onboarding_scan,
            onboarding_apply,
            onboarding_skip,
        ])
        .run(tauri::generate_context!())
        .expect("fatal: error while running tauri application");
}

/// One-time application setup: initialise state, reconcile the OS autostart
/// registration, build the tray, and show/hide the window per the saved
/// preferences (or the `--hidden` autostart launch flag).
fn setup_app(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let state = state::AppState::init(app.handle())
        .map_err(|e| format!("fatal: failed to initialise AppState: {e}"))?;
    // Snapshot the launch-relevant preferences before the state is moved into
    // management (defaults on first run / a read failure).
    let launch_prefs = state.settings.get().unwrap_or_default();
    // Resume Smart Switching before the state is moved into management, when the
    // user opted to start watching on launch. Best-effort — a failure here must
    // never block app startup.
    let orchestrator = state.identity_switch.clone();
    app.manage(state);
    let _ = orchestrator.maybe_start_on_launch();
    // Pin every tracked repository's resolved identity and per-repo HTTPS
    // credential, independent of whether Smart Switching starts — so existing
    // assignments authenticate as their assigned account after an upgrade.
    // Best-effort: a pin failure must never block app startup.
    orchestrator.pin_all();

    // Bring the OS autostart registration in line with the saved preference, then
    // show/hide the window and build the tray so the app can live in the
    // background like any other desktop app.
    sync_autostart(app.handle(), launch_prefs.launch_at_startup);
    build_tray(app.handle())?;
    let start_hidden = launch_prefs.start_minimized || std::env::args().any(|a| a == "--hidden");
    if let Some(window) = app.get_webview_window("main") {
        if start_hidden {
            let _ = window.hide();
        } else {
            let _ = window.show();
        }
    }

    #[cfg(debug_assertions)]
    api::serve_docs();
    Ok(())
}

/// Register or unregister `GitPersona` from the OS login autostart list to match
/// the saved preference. Best-effort: a failure here must never break startup or
/// the settings-save flow, so the result is intentionally discarded.
pub(crate) fn sync_autostart(app: &tauri::AppHandle, enabled: bool) {
    use tauri_plugin_autostart::ManagerExt;
    let manager = app.autolaunch();
    let _ = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
}

/// Build the persistent system-tray icon with a Show/Quit menu. Left-clicking
/// the icon restores the window; Quit exits outright, bypassing the
/// close-to-tray guard.
fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show GitPersona", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit GitPersona", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .tooltip("GitPersona")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    builder.build(app)?;
    Ok(())
}

/// Restore and focus the main window (from the tray or a re-launch).
fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod export_bindings {
    use crate::domain::{
        audit::AuditEntry,
        bootstrap::{AnnouncementAction, AnnouncementInfo, BootstrapData},
        credential::{Credential, Protocol},
        identity::Identity,
        identity_switch::{SmartSwitchStatus, SwitchEvent, SwitchStatus},
        onboarding::{DetectedCredential, DetectedIdentity, DetectedSshKey, SystemScan},
        profile::Profile,
        repo::{Repo, RepoGroup, ScanProgress},
        rule::{
            Rule, RuleCondition, RuleMatch, RuleOperator, RulePreviewInput, RulePreviewResult,
            RuleSubject, RuleSummary,
        },
        settings::{AppSettings, SmartSwitchingSettings, Theme},
        ssh::{SshAlgorithm, SshKey},
    };
    use ts_rs::TS;

    #[test]
    fn export_ts_bindings() {
        BootstrapData::export_all_to("../src/ipc/").unwrap();
        AnnouncementInfo::export_all_to("../src/ipc/").unwrap();
        AnnouncementAction::export_all_to("../src/ipc/").unwrap();
        Identity::export_all_to("../src/ipc/").unwrap();
        Profile::export_all_to("../src/ipc/").unwrap();
        Repo::export_all_to("../src/ipc/").unwrap();
        RepoGroup::export_all_to("../src/ipc/").unwrap();
        ScanProgress::export_all_to("../src/ipc/").unwrap();
        AuditEntry::export_all_to("../src/ipc/").unwrap();
        AppSettings::export_all_to("../src/ipc/").unwrap();
        Theme::export_all_to("../src/ipc/").unwrap();
        SmartSwitchingSettings::export_all_to("../src/ipc/").unwrap();
        SshKey::export_all_to("../src/ipc/").unwrap();
        SshAlgorithm::export_all_to("../src/ipc/").unwrap();
        Credential::export_all_to("../src/ipc/").unwrap();
        Protocol::export_all_to("../src/ipc/").unwrap();
        SmartSwitchStatus::export_all_to("../src/ipc/").unwrap();
        SwitchEvent::export_all_to("../src/ipc/").unwrap();
        SystemScan::export_all_to("../src/ipc/").unwrap();
        DetectedIdentity::export_all_to("../src/ipc/").unwrap();
        DetectedSshKey::export_all_to("../src/ipc/").unwrap();
        DetectedCredential::export_all_to("../src/ipc/").unwrap();
        SwitchStatus::export_all_to("../src/ipc/").unwrap();
        Rule::export_all_to("../src/ipc/").unwrap();
        RuleCondition::export_all_to("../src/ipc/").unwrap();
        RuleSubject::export_all_to("../src/ipc/").unwrap();
        RuleOperator::export_all_to("../src/ipc/").unwrap();
        RuleMatch::export_all_to("../src/ipc/").unwrap();
        RulePreviewInput::export_all_to("../src/ipc/").unwrap();
        RulePreviewResult::export_all_to("../src/ipc/").unwrap();
        RuleSummary::export_all_to("../src/ipc/").unwrap();
    }
}
