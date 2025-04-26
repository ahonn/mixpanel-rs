use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime, State,
};

mod commands;
mod error;
mod people;
mod persistence;
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
        let token = self.token;
        let api_host = self.api_host;

        PluginBuilder::<R>::new("mixpanel")
            .invoke_handler(tauri::generate_handler![
                commands::register,
                commands::register_once,
                commands::unregister,
            ])
            .setup(move |app_handle, _api| {
                match MixpanelState::new(&token, api_host.clone(), app_handle) {
                    Ok(state) => {
                        app_handle.manage(state);
                        Ok(())
                    }
                    Err(e) => {
                        panic!("Failed to initialize Mixpanel: {:?}", e);
                    }
                }
            })
            .build()
    }
}
