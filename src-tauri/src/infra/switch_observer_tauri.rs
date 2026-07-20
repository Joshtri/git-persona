use crate::domain::identity_switch::{SmartSwitchStatus, SwitchEvent};
use crate::domain::ports::SwitchObserver;
use tauri::Emitter;

const SWITCH_EVENT: &str = "smart-switch";
const STATUS_EVENT: &str = "smart-switch-status";

/// Emits orchestrator events to the frontend over Tauri's event bus. Payloads
/// carry display metadata only — never secrets or key material.
pub(crate) struct TauriSwitchObserver {
    app: tauri::AppHandle,
}

impl TauriSwitchObserver {
    pub(crate) fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }
}

impl SwitchObserver for TauriSwitchObserver {
    fn switched(&self, event: &SwitchEvent) {
        let _ = self.app.emit(SWITCH_EVENT, event);
    }

    fn status_changed(&self, status: &SmartSwitchStatus) {
        let _ = self.app.emit(STATUS_EVENT, status);
    }
}
