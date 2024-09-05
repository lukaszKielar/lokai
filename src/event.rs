use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::interval;

use crate::{app::AppResult, models::Message};

// TODO: remove events I won't use
// TODO: add new events
#[derive(Clone, Debug)]
pub enum Event {
    TerminalTick,
    Key(KeyEvent),
    Inference(Message, bool),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    event_rx: mpsc::UnboundedReceiver<Event>,
    join_handle: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(
        tick_rate: u64,
        event_tx: mpsc::UnboundedSender<Event>,
        event_rx: mpsc::UnboundedReceiver<Event>,
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
                        event_tx.send(Event::TerminalTick).unwrap();
                    }
                    Some(Ok(evt)) = crossterm_event => {
                        if let CrosstermEvent::Key(key) = evt {
                            if key.kind == KeyEventKind::Press {
                                event_tx.send(Event::Key(key)).unwrap();
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
