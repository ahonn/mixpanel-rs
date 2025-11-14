pub use mixpanel_rs::Config;
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

    fn try_mixpanel(&self) -> Option<State<'_, MixpanelState>>;
}

impl<R: Runtime> MixpanelExt for tauri::AppHandle<R> {
    fn mixpanel(&self) -> State<'_, MixpanelState> {
        self.state::<MixpanelState>()
    }

    fn try_mixpanel(&self) -> Option<State<'_, MixpanelState>> {
        self.try_state::<MixpanelState>()
    }
}

pub struct Builder {
    token: String,
    config: Option<Config>,
}

impl Builder {
    pub fn new(token: impl Into<String>, config: Option<Config>) -> Self {
        Self {
            token: token.into(),
            config,
        }
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        let token = self.token;
        let config = self.config;

        PluginBuilder::<R>::new("mixpanel")
            .invoke_handler(tauri::generate_handler![
                commands::register,
                commands::register_once,
                commands::unregister,
                commands::identify,
                commands::alias,
                commands::track,
                commands::get_distinct_id,
                commands::get_property,
                commands::reset,
                commands::time_event,
                commands::set_group,
                commands::add_group,
                commands::remove_group,
                commands::people_set,
                commands::people_set_once,
                commands::people_unset,
                commands::people_increment,
                commands::people_append,
                commands::people_remove,
                commands::people_union,
                commands::people_delete_user,
            ])
            .setup(
                move |app_handle, _api| match MixpanelState::new(app_handle, &token, config) {
                    Ok(state) => {
                        app_handle.manage(state);
                        Ok(())
                    }
                    Err(e) => {
                        panic!("Failed to initialize Mixpanel: {:?}", e);
                    }
                },
            )
            .build()
    }
}
