use cpal::traits::{EventLoopTrait, HostTrait};
use cpal::{Format, SampleFormat, SampleRate};
use cpal::{StreamData, UnknownTypeOutputBuffer};
use ringbuf::{Consumer, RingBuffer};
use std::io::Read;
use std::net::TcpStream;
use std::thread::spawn;

pub(crate) struct ConnectionHandler;

impl ConnectionHandler {
    #[allow(unused_must_use)]
    /// We allow not using a result here since writing to the buffer
    /// can fail in certain cases (which we should feel free to ignore, but yeah...)
    pub fn handle(mut stream: TcpStream) {
        // create ringbuffer
        let ring = RingBuffer::new(8192);
        let (mut producer, consumer) = ring.split();

        // create buffer for stream
        let mut buffer = [0; 8192];

        // give the buffer a headstart
        stream.read(&mut buffer).unwrap();
        buffer
            .chunks(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .for_each(|sample| producer.push(sample).unwrap());

        // And spawn the thread responsible for playing audio!
        Self::spawn_audio_thread(consumer);

        let mut bytes_read = 1;
        while bytes_read > 0 {
            bytes_read = stream.read(&mut buffer).unwrap();
            buffer
                .chunks(2)
                .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                .for_each(|sample| {
                    producer.push(sample);
                });
        }
    }

    fn spawn_audio_thread(mut consumer: Consumer<i16>) {
        let host = cpal::default_host();
        let event_loop = host.event_loop();

        let device = host
            .default_output_device()
            .expect("no output device available");

        let format = Format {
            channels: 2,
            sample_rate: SampleRate(44100),
            data_type: SampleFormat::I16,
        };

        let stream_id = event_loop.build_output_stream(&device, &format).unwrap();

        event_loop
            .play_stream(stream_id)
            .expect("failed to play_stream");

        spawn(move || {
            event_loop.run(move |stream_id, stream_result| {
                let stream_data = match stream_result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                        return;
                    }
                };

                match stream_data {
                    StreamData::Output {
                        buffer: UnknownTypeOutputBuffer::I16(mut buffer),
                    } => {
                        for elem in buffer.iter_mut() {
                            *elem = match consumer.pop() {
                                Some(val) => val,
                                None => 0,
                            };
                        }
                    }
                    StreamData::Output {
                        buffer: UnknownTypeOutputBuffer::F32(mut buffer),
                    } => {
                        for elem in buffer.iter_mut() {
                            *elem = 0.0;
                        }
                    }
                    StreamData::Output {
                        buffer: UnknownTypeOutputBuffer::U16(mut buffer),
                    } => {
                        for elem in buffer.iter_mut() {
                            *elem = 0;
                        }
                    }
                    _ => (),
                }
            });
        });
    }
}
