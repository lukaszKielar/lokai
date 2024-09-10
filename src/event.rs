use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
    time::interval,
};

use crate::{models::Message, AppResult};

#[derive(Clone, Debug)]
pub enum InferenceType {
    Streaming,
    NonStreaming,
}

#[derive(Clone, Debug)]
pub enum Event {
    TerminalTick,
    Key(KeyEvent),
    Inference(Message, InferenceType),
    ChatBottomScroll,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    event_rx: UnboundedReceiver<Event>,
    join_handle: JoinHandle<()>,
}

impl EventHandler {
    pub fn new(
        tick_rate: u64,
        event_tx: UnboundedSender<Event>,
        event_rx: UnboundedReceiver<Event>,
    ) -> Self {
        let join_handle = tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick = interval(Duration::from_millis(tick_rate));
            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    _ = event_tx.closed() => {
                        break;
                    }
                    _ = tick_delay => {
                        // ignore any tick errors, ideally log that
                        event_tx.send(Event::TerminalTick).expect("Cannot send tick event");
                    }
                    Some(Ok(evt)) = crossterm_event => {
                        if let CrosstermEvent::Key(key) = evt {
                            if key.kind == KeyEventKind::Press {
                                event_tx.send(Event::Key(key)).expect("Cannot send key event");
                            }
                        }
                    }
                };
            }
        });
        Self {
            event_rx,
            join_handle,
        }
    }

    pub async fn next(&mut self) -> AppResult<Event> {
        self.event_rx
            .recv()
            .await
            .ok_or(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "This is an IO error",
            )))
    }
}
