use std::sync::Arc;

use gtk::gio::Settings;

use crate::api::InvidiousClient;

#[derive(Debug)]
pub struct AppModel {
    invidious: Arc<InvidiousClient>,
    settings: Arc<Settings>,
}

impl AppModel {
    pub fn new(invidious: InvidiousClient, settings: Settings) -> Self {
        AppModel {
            invidious: Arc::new(invidious),
            settings: Arc::new(settings),
        }
    }

    pub fn invidious(&self) -> Arc<InvidiousClient> {
        self.invidious.clone()
    }

    pub fn settings(&self) -> Arc<Settings> {
        self.settings.clone()
    }
}
