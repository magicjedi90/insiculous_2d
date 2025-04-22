use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;

/// Handles audio playback.
pub struct Audio {
    _stream: OutputStream,
    sink: Sink,
}

impl Audio {
    /// Creates a new audio player.
    pub fn new() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        Audio { _stream, sink }
    }

    /// Plays the audio file at the given path.
    pub fn play(&self, asset_path: &str) {
        let file = File::open(asset_path).unwrap();
        let source = Decoder::new(std::io::BufReader::new(file)).unwrap();
        self.sink.append(source);
    }
}
