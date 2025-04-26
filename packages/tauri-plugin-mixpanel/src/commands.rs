use serde_json::Value;
use tauri::{command, ipc::InvokeError, AppHandle, Manager, Runtime};

use crate::state::MixpanelState;

type Result<T> = std::result::Result<T, InvokeError>;

#[command]
pub async fn register<R: Runtime>(
    properties: Value,
    options: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<crate::state::MixpanelState>();
    state
        .register(properties, options)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn register_once<R: Runtime>(
    properties: Value,
    default_value: Option<Value>,
    options: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<crate::state::MixpanelState>();
    state
        .register_once(properties, default_value, options)
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn unregister<R: Runtime>(
    property_name: String,
    options: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<crate::state::MixpanelState>();
    state
        .unregister(&property_name, options)
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub fn get_property<R: Runtime>(property_name: String, app_handle: AppHandle<R>) -> Result<Option<Value>> {
    let state = app_handle.state::<MixpanelState>();
    Ok(state.get_property(&property_name))
}

#[command]
pub fn time_event<R: Runtime>(event_name: String, app_handle: AppHandle<R>) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state.time_event(&event_name);
    Ok(())
}

#[command]
pub async fn set_group<R: Runtime>(
    group_key: String,
    group_ids: Value,
    options: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .set_group(&group_key, group_ids, options)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn add_group<R: Runtime>(
    group_key: String,
    group_id: Value,
    options: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .add_group(&group_key, group_id, options)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn remove_group<R: Runtime>(
    group_key: String,
    group_id: Value,
    options: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .remove_group(&group_key, group_id, options)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn identify<R: Runtime>(distinct_id: String, app_handle: AppHandle<R>) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .identify(distinct_id)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn alias<R: Runtime>(
    alias: String,
    original: Option<String>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .alias(alias, original)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub fn reset<R: Runtime>(app_handle: AppHandle<R>) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state.reset().map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn track<R: Runtime>(
    event_name: String,
    properties: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .track(event_name, properties)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub fn get_distinct_id<R: Runtime>(app_handle: AppHandle<R>) -> Result<Option<String>> {
    let state = app_handle.state::<MixpanelState>();
    Ok(state.get_distinct_id())
}

// --- People Commands ---

#[command]
pub async fn people_set<R: Runtime>(
    prop: Value,
    to: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .people
        .set(prop, to)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn people_set_once<R: Runtime>(
    prop: Value,
    to: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .people
        .set_once(prop, to)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn people_unset<R: Runtime>(prop: Value, app_handle: AppHandle<R>) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .people
        .unset(prop)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn people_increment<R: Runtime>(
    prop: Value,
    by: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .people
        .increment(prop, by)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn people_append<R: Runtime>(
    list_name: Value,
    value: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .people
        .append(list_name, value)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn people_remove<R: Runtime>(
    list_name: Value,
    value: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .people
        .remove(list_name, value)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn people_union<R: Runtime>(
    list_name: Value,
    values: Option<Value>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .people
        .union(list_name, values)
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}

#[command]
pub async fn people_delete_user<R: Runtime>(app_handle: AppHandle<R>) -> Result<()> {
    let state = app_handle.state::<MixpanelState>();
    state
        .people
        .delete_user()
        .await
        .map_err(InvokeError::from_error)?;
    Ok(())
}
