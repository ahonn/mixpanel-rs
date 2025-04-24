use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Mixpanel;
#[cfg(mobile)]
use mobile::Mixpanel;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the mixpanel APIs.
pub trait MixpanelExt<R: Runtime> {
  fn mixpanel(&self) -> &Mixpanel<R>;
}

impl<R: Runtime, T: Manager<R>> crate::MixpanelExt<R> for T {
  fn mixpanel(&self) -> &Mixpanel<R> {
    self.state::<Mixpanel<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("mixpanel")
    .invoke_handler(tauri::generate_handler![commands::ping])
    .setup(|app, api| {
      #[cfg(mobile)]
      let mixpanel = mobile::init(app, api)?;
      #[cfg(desktop)]
      let mixpanel = desktop::init(app, api)?;
      app.manage(mixpanel);
      Ok(())
    })
    .build()
}
