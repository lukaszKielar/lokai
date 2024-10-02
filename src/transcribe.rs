use std::time::Duration;

use futures::StreamExt;
use kalosm::sound::TextStream;
use kalosm_sound::{
    DenoisedExt, MicInput, TranscribeChunkedAudioStreamExt, VoiceActivityStreamExt, Whisper,
};
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};
use tracing::info;

use crate::event::Event;

pub struct Transcriber {
    join_handle: JoinHandle<()>,
}

impl Transcriber {
    pub fn new(event_tx: UnboundedSender<Event>, whisper: Whisper) -> Self {
        let mic_input = MicInput::default();
        let mic_stream = mic_input.stream().expect("Cannot create MicStream");

        let join_handle = tokio::spawn(async move {
            let voice_stream = mic_stream
                .denoise_and_detect_voice_activity()
                .rechunk_voice_activity();
            let text_stream = voice_stream.transcribe(whisper);
            let mut sentences = text_stream.sentences();

            // TODO: use tokio::select!
            loop {
                match sentences.next().await {
                    Some(sentence) => {
                        info!("sentence: {:?}", sentence);
                        // TODO: handle error
                        event_tx.send(Event::PromptTranscription(sentence.to_string()));
                    }
                    None => tokio::time::sleep(Duration::from_secs(2)).await,
                }
            }
        });

        Self { join_handle }
    }
}
