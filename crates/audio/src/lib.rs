use rodio::{OutputStream, Sink, Decoder};
use std::io::Cursor;

pub struct Audio {
    _stream: OutputStream,
    sink: Sink,
}
impl Audio {
    pub fn new() -> Self {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&handle).unwrap();
        Self { _stream, sink }
    }
    pub fn play(&self, bytes: &[u8]) {
        let source = Decoder::new(Cursor::new(bytes.to_vec())).unwrap();
        self.sink.append(source);
    }
}
