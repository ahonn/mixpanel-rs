use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime, State,
};

mod commands;
mod error;
mod state;

use state::MixpanelState;

pub trait MixpanelExt {
    fn mixpanel(&self) -> State<'_, MixpanelState>;
}

impl<R: Runtime> MixpanelExt for tauri::AppHandle<R> {
    fn mixpanel(&self) -> State<'_, MixpanelState> {
        self.state::<MixpanelState>()
    }
}

pub struct Builder {
    token: String,
    api_host: Option<String>,
}

impl Builder {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            api_host: None,
        }
    }

    pub fn api_host(mut self, api_host: impl Into<String>) -> Self {
        self.api_host = Some(api_host.into());
        self
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        let mixpanel_state = MixpanelState::new(self.token, self.api_host);

        PluginBuilder::new("mixpanel")
            .invoke_handler(tauri::generate_handler![
                commands::track,
                commands::identify,
                commands::alias,
            ])
            .setup(move |app_handle, _api| {
                app_handle.manage(mixpanel_state);
                Ok(())
            })
            .build()
    }
}
