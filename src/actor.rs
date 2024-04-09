use tokio::sync::mpsc;

use crate::layout::{LayoutManager, SpiDisplay};
use crate::types::*;

pub struct Actor<'a> {
    receiver: mpsc::Receiver<ActorMessage>,
    manager: LayoutManager<'a>,
}

pub enum ActorMessage {
    GetKey(KeyMap),
    UpdateScreen(Box<SpiDisplay>),
}

impl Actor<'_> {
    pub fn new(receiver: mpsc::Receiver<ActorMessage>) -> Self {
        Actor {
            receiver,
            manager: LayoutManager::new(),
        }
    }

    pub fn handle_message(&mut self, msg: ActorMessage) {
        match msg {
            ActorMessage::GetKey(key) => {
                self.manager.input(key);
            }
            ActorMessage::UpdateScreen(mut display) => {
                self.manager.draw(display.as_mut());
            }
            _ => {}
        }
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg);
        }
    }
}
