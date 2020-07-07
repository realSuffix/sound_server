use tokio::{net::TcpListener, stream::StreamExt};
use rodio::buffer::SamplesBuffer;
use rodio::Source;
use tokio::io::AsyncReadExt;
use wavy::{S16LEx2, Player, Recorder};
use tokio::time::Duration;
use std::cell::RefCell;

struct Shared {
    /// A stereo audio buffer.
    buffer: Vec<S16LEx2>,
}

#[tokio::main]
async fn main() {
    let mut listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let mut incoming = listener.incoming();
    let device = rodio::default_output_device().unwrap();

    while let Some(stream) = incoming.next().await {
        match stream {
            Ok(mut stream) => {
                println!("new client!");
                let mut bytes_read = 1;
                let mut buffer = vec![0; 33554432].into_boxed_slice();

                while bytes_read > 0 {
                    // as long as there are still bytes, we want to read them
                    bytes_read = stream.read(&mut buffer[..]).await.unwrap();
                    let res: Vec<i16> = buffer
                        .chunks(2)
                        .into_iter()
                        .map::<[u8; 2], _>(|chunk| [chunk[0], chunk[1]])
                        .map(|chunk| i16::from_le_bytes(chunk))
                        .collect();
                    let res = SamplesBuffer::new(2, 44100, res);
                    rodio::play_raw(&device, res.convert_samples());
                }
                println!("done");

            }
            Err(e) => { /* connection failed */ }
        }
    }
}
