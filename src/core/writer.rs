use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
    sync::{mpsc, oneshot},
};
use tokio_rustls::server::TlsStream;
use tracing::debug;

use super::client::{Session, SessionData};

const IPC_CHANNEL_BUFFER: usize = 128;

pub enum Event {
    Stream(WriteHalf<TcpStream>),
    StreamTls(WriteHalf<TlsStream<TcpStream>>),
    Bytes(Vec<u8>),
    Upgrade(oneshot::Sender<Event>),
}

pub fn spawn_writer() -> mpsc::Sender<Event> {
    let (tx, mut rx) = mpsc::channel::<Event>(IPC_CHANNEL_BUFFER);
    tokio::spawn(async move {
        let mut stream = rx.recv().await.unwrap();
        'outer: loop {
            match stream {
                Event::Stream(mut stream_tx) => {
                    while let Some(event) = rx.recv().await {
                        match event {
                            Event::Bytes(bytes) => {
                                /*let tmp = "dd";
                                println!(
                                    "-> {:?}",
                                    String::from_utf8_lossy(
                                        &bytes[..std::cmp::min(bytes.len(), 100)]
                                    )
                                );*/

                                if let Err(err) = stream_tx.write_all(&bytes).await {
                                    debug!("Failed to write to stream: {}", err);
                                    break 'outer;
                                }
                            }
                            Event::Upgrade(channel) => {
                                if channel.send(Event::Stream(stream_tx)).is_err() {
                                    debug!("Failed to send stream.");
                                    break 'outer;
                                }
                                if let Some(stream_) = rx.recv().await {
                                    stream = stream_;
                                    continue 'outer;
                                } else {
                                    break 'outer;
                                }
                            }
                            _ => {
                                stream = event;
                                continue 'outer;
                            }
                        }
                    }
                    break 'outer;
                }
                Event::StreamTls(mut stream_tx) => {
                    while let Some(event) = rx.recv().await {
                        match event {
                            Event::Bytes(bytes) => {
                                if let Err(err) = stream_tx.write_all(&bytes).await {
                                    debug!("Failed to write to stream: {}", err);
                                    break 'outer;
                                }
                            }
                            _ => {
                                stream = event;
                                continue 'outer;
                            }
                        }
                    }
                    break 'outer;
                }
                _ => unreachable!(),
            }
        }
    });
    tx
}

impl Session {
    pub async fn write_bytes(&self, bytes: Vec<u8>) -> Result<(), ()> {
        if let Err(err) = self.writer.send(Event::Bytes(bytes)).await {
            debug!("Failed to send bytes: {}", err);
            Err(())
        } else {
            Ok(())
        }
    }
}

impl SessionData {
    pub async fn write_bytes(&self, bytes: Vec<u8>) -> bool {
        if let Err(err) = self.writer.send(Event::Bytes(bytes)).await {
            debug!("Failed to send bytes: {}", err);
            false
        } else {
            true
        }
    }
}
