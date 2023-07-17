use std::sync::Arc;

use crate::api::InvidiousClient;

#[derive(Debug)]
pub struct AppModel {
    invidious: Arc<InvidiousClient>,
}

impl AppModel {
    pub fn new(invidious: Arc<InvidiousClient>) -> Self {
        AppModel { invidious }
    }

    pub fn invidious(&self) -> Arc<InvidiousClient> {
        self.invidious.clone()
    }
}
