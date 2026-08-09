use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
// Each bool is an independent, persisted user preference mapping 1:1 to a toggle
// in the Settings UI, so collapsing them into a state type would obscure them
// and break the wire format.
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct AppSettings {
    pub(crate) theme: Theme,
    pub(crate) show_audit_log: bool,
    pub(crate) auto_scan_repos: bool,
    /// Smart Identity Switching (Sprint 6). `#[serde(default)]` keeps settings
    /// files written before this field was introduced loadable.
    #[serde(default)]
    pub(crate) smart_switching: SmartSwitchingSettings,
    /// `true` once the first-run onboarding wizard has been completed or skipped.
    /// `#[serde(default)]` (to `false`) makes existing users see onboarding once,
    /// then keeps them from being prompted again.
    #[serde(default)]
    pub(crate) onboarded: bool,
    /// Register `GitPersona` to start automatically when the user logs in.
    /// Applied by (de)registering the OS autostart entry when settings are saved.
    #[serde(default)]
    pub(crate) launch_at_startup: bool,
    /// Start hidden in the system tray instead of showing the main window. Also
    /// implied when the app is launched by the autostart entry (`--hidden`).
    #[serde(default)]
    pub(crate) start_minimized: bool,
    /// Keep running in the tray when the main window is closed, instead of
    /// quitting, so Smart Switching keeps watching in the background.
    #[serde(default)]
    pub(crate) close_to_tray: bool,
}

/// Configuration for the Smart Identity Switching engine. Read live on every
/// activation event, so a change takes effect on the next repository switch.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(default)]
// A user-facing settings record: each flag is an independent, persisted toggle
// that maps 1:1 to a switch in the Settings UI, so a state machine would only
// obscure it and break the wire format.
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct SmartSwitchingSettings {
    /// Master switch: when off, the workspace watcher does not run.
    pub(crate) enabled: bool,
    /// Reserved — SSH identities are wired globally via the managed `~/.ssh/config`
    /// block, so they follow the applied profile automatically. Surfaced as a
    /// toggle for parity and a future per-stage gate.
    pub(crate) auto_ssh: bool,
    /// Reserved — HTTPS credentials are switched as part of the atomic profile
    /// apply. Surfaced as a toggle for parity and a future per-stage gate.
    pub(crate) auto_credential: bool,
    /// Emit an in-app notification after an automatic switch.
    pub(crate) show_notification: bool,
    /// Require explicit confirmation before applying an automatic switch.
    pub(crate) confirm_before_switch: bool,
    /// Start watching automatically when the application launches.
    pub(crate) start_on_launch: bool,
}

impl Default for SmartSwitchingSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_ssh: true,
            auto_credential: true,
            show_notification: true,
            confirm_before_switch: false,
            start_on_launch: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub(crate) enum Theme {
    Dark,
    Light,
    System,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            show_audit_log: true,
            auto_scan_repos: false,
            smart_switching: SmartSwitchingSettings::default(),
            onboarded: false,
            launch_at_startup: false,
            start_minimized: false,
            close_to_tray: false,
        }
    }
}
