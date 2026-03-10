use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use super::InternalTcpEvent;
use crate::core::providers::network_types::{Channel, ConnectionId, DisconnectReason};

const READ_TIMEOUT: Duration = Duration::from_millis(10);
const MAX_MESSAGE_SIZE: usize = 16_777_215;

pub(super) enum ReadOutcome {
    Frame(Channel, Vec<u8>),
    Timeout,
    Closed,
}

pub(super) fn configure_stream(stream: &TcpStream) {
    let _ = stream.set_nodelay(true);
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
}

pub(super) fn read_frame(stream: &mut TcpStream) -> Result<ReadOutcome, String> {
    let mut len_buf = [0u8; 4];
    match stream.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(ref e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ) =>
        {
            return Ok(ReadOutcome::Timeout);
        }
        Err(ref e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::UnexpectedEof
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::BrokenPipe
            ) =>
        {
            return Ok(ReadOutcome::Closed);
        }
        Err(e) => return Err(format!("read length: {e}")),
    }

    let frame_len = u32::from_be_bytes(len_buf) as usize;
    if frame_len == 0 || frame_len > MAX_MESSAGE_SIZE + 1 {
        return Err(format!("invalid frame length: {frame_len}"));
    }

    let mut frame = vec![0u8; frame_len];
    match stream.read_exact(&mut frame) {
        Ok(()) => Ok(ReadOutcome::Frame(Channel(frame[0]), frame[1..].to_vec())),
        Err(ref e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::UnexpectedEof
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::BrokenPipe
            ) =>
        {
            Ok(ReadOutcome::Closed)
        }
        Err(ref e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ) =>
        {
            Ok(ReadOutcome::Timeout)
        }
        Err(e) => Err(format!("read frame: {e}")),
    }
}

pub(super) fn spawn_io_thread(
    cid: ConnectionId,
    mut stream: TcpStream,
    event_tx: mpsc::Sender<InternalTcpEvent>,
    running: Arc<AtomicBool>,
) -> mpsc::Sender<Vec<u8>> {
    let (write_tx, write_rx) = mpsc::channel::<Vec<u8>>();
    thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            match read_frame(&mut stream) {
                Ok(ReadOutcome::Frame(channel, data)) => {
                    let _ = event_tx.send(InternalTcpEvent::Received(cid, channel, data));
                }
                Ok(ReadOutcome::Timeout) => {}
                Ok(ReadOutcome::Closed) => {
                    let _ = event_tx.send(InternalTcpEvent::Disconnected(
                        cid,
                        DisconnectReason::RemoteClose,
                    ));
                    break;
                }
                Err(err) => {
                    let _ = event_tx.send(InternalTcpEvent::Error(cid, err.clone()));
                    let _ = event_tx.send(InternalTcpEvent::Disconnected(
                        cid,
                        DisconnectReason::Error(err),
                    ));
                    break;
                }
            }

            loop {
                match write_rx.try_recv() {
                    Ok(frame) => {
                        if let Err(e) = stream.write_all(&frame) {
                            let _ =
                                event_tx.send(InternalTcpEvent::Error(cid, format!("write: {e}")));
                            let _ = event_tx.send(InternalTcpEvent::Disconnected(
                                cid,
                                DisconnectReason::Error(e.to_string()),
                            ));
                            return;
                        }
                    }
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        let _ = stream.shutdown(Shutdown::Both);
                        return;
                    }
                }
            }
        }
    });
    write_tx
}
