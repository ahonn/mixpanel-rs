use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::MixpanelExt;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.mixpanel().ping(payload)
}
