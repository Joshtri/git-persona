use crate::{
    domain::rule::{
        Rule, RuleCondition, RuleOperator, RulePreviewInput, RulePreviewResult, RuleSubject,
        RuleSummary,
    },
    error::AppError,
    state::AppState,
};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub(crate) async fn rule_list(state: State<'_, AppState>) -> Result<Vec<Rule>, AppError> {
    state.rules.list()
}

#[tauri::command]
pub(crate) async fn rule_get(id: Uuid, state: State<'_, AppState>) -> Result<Rule, AppError> {
    state.rules.get(id)
}

#[tauri::command]
pub(crate) async fn rule_create(
    name: String,
    subject: RuleSubject,
    operator: RuleOperator,
    value: String,
    target_profile_id: Uuid,
    state: State<'_, AppState>,
) -> Result<Rule, AppError> {
    let condition = RuleCondition {
        subject,
        operator,
        value,
    };
    state.rules.create(&name, condition, target_profile_id)
}

#[tauri::command]
pub(crate) async fn rule_update(
    id: Uuid,
    name: String,
    subject: RuleSubject,
    operator: RuleOperator,
    value: String,
    target_profile_id: Uuid,
    state: State<'_, AppState>,
) -> Result<Rule, AppError> {
    let condition = RuleCondition {
        subject,
        operator,
        value,
    };
    state.rules.update(id, &name, condition, target_profile_id)
}

#[tauri::command]
pub(crate) async fn rule_delete(id: Uuid, state: State<'_, AppState>) -> Result<(), AppError> {
    state.rules.delete(id)
}

#[tauri::command]
pub(crate) async fn rule_duplicate(id: Uuid, state: State<'_, AppState>) -> Result<Rule, AppError> {
    state.rules.duplicate(id)
}

#[tauri::command]
pub(crate) async fn rule_set_enabled(
    id: Uuid,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<Rule, AppError> {
    state.rules.set_enabled(id, enabled)
}

#[tauri::command]
pub(crate) async fn rule_set_all_enabled(
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<Vec<Rule>, AppError> {
    state.rules.set_all_enabled(enabled)
}

#[tauri::command]
pub(crate) async fn rule_reorder(
    ordered_ids: Vec<Uuid>,
    state: State<'_, AppState>,
) -> Result<Vec<Rule>, AppError> {
    state.rules.reorder(&ordered_ids)
}

#[tauri::command]
pub(crate) async fn rule_preview(
    input: RulePreviewInput,
    state: State<'_, AppState>,
) -> Result<RulePreviewResult, AppError> {
    state.rules.preview(&input)
}

#[tauri::command]
pub(crate) async fn rule_summary(state: State<'_, AppState>) -> Result<RuleSummary, AppError> {
    state.rules.summary()
}

/// Serialize all rules and write them to `path` (chosen via the dialog plugin).
#[tauri::command]
pub(crate) async fn rule_export(path: String, state: State<'_, AppState>) -> Result<(), AppError> {
    let json = state.rules.export()?;
    std::fs::write(&path, json).map_err(|e| AppError::Io(e.to_string()))
}

/// Read a rules file at `path` and import it. `replace` wipes the current set.
#[tauri::command]
pub(crate) async fn rule_import(
    path: String,
    replace: bool,
    state: State<'_, AppState>,
) -> Result<Vec<Rule>, AppError> {
    let json = std::fs::read_to_string(&path).map_err(|e| AppError::Io(e.to_string()))?;
    state.rules.import(&json, replace)
}
