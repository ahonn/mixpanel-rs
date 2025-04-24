use serde_json::Value;
use tauri::{command, ipc::InvokeError, AppHandle, Manager, Runtime};

type Result<T> = std::result::Result<T, InvokeError>;

#[command]
pub async fn track<R: Runtime>(
    event_name: String,
    properties: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<crate::state::MixpanelState>();
    state.track(&event_name, properties).await?;
    Ok(())
}

#[command]
pub async fn identify<R: Runtime>(
    distinct_id: String,
    properties: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<crate::state::MixpanelState>();
    state.identify(&distinct_id, properties).await?;
    Ok(())
}

#[command]
pub async fn alias<R: Runtime>(
    distinct_id: String,
    alias: String,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<crate::state::MixpanelState>();
    state.alias(&distinct_id, &alias).await?;
    Ok(())
}

