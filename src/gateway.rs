use tokio::sync::mpsc;

use crate::protocol::{EngineToUi, UiToEngine};

pub struct Gateway {
    ui_to_engine: mpsc::Sender<UiToEngine>,
    engine_to_ui: mpsc::Receiver<EngineToUi>,
}

impl Gateway {
    pub fn new(
        ui_to_engine: mpsc::Sender<UiToEngine>,
        engine_to_ui: mpsc::Receiver<EngineToUi>,
    ) -> Self {
        Self {
            ui_to_engine,
            engine_to_ui,
        }
    }

    pub async fn send(&self, msg: UiToEngine) {
        let _ = self.ui_to_engine.send(msg).await;
    }

    pub async fn recv(&mut self) -> Option<EngineToUi> {
        self.engine_to_ui.recv().await
    }
}
