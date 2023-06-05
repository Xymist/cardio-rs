// Conversion of [Cardio](https://github.com/Xe/x/blob/master/cardio/cardio.go) to Rust
// Original by... Mara? Christine? Xe? The individual who wrote the original code is not clear about
// their preferred name, so just see the GitHub link above.

use std::{
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    time::Duration,
};

const DEFAULT_DELAY: Duration = Duration::from_millis(500);

pub struct Heartbeat {
    heartbeat: Receiver<()>,
    av: Sender<HeartbeatMessage>,
}

pub enum HeartbeatMessage {
    Slower,
    Faster,
    Die,
}

impl Heartbeat {
    pub fn new(min: Duration, max: Duration) -> Heartbeat {
        let (hb_tx, hb_rx) = channel();
        let (av_tx, av_rx) = channel();

        std::thread::spawn(move || hb_thread(hb_tx, av_rx, min, max));

        Heartbeat {
            heartbeat: hb_rx,
            av: av_tx,
        }
    }

    pub fn faster(&self) {
        self.av.send(HeartbeatMessage::Faster).unwrap();
    }

    pub fn slower(&self) {
        self.av.send(HeartbeatMessage::Slower).unwrap();
    }

    pub fn die(&self) {
        self.av.send(HeartbeatMessage::Die).unwrap();
    }

    pub fn try_recv(&self) -> Result<(), TryRecvError> {
        self.heartbeat.try_recv()
    }
}

impl Drop for Heartbeat {
    fn drop(&mut self) {
        self.die();
    }
}

fn hb_thread(hb_tx: Sender<()>, av_rx: Receiver<HeartbeatMessage>, min: Duration, max: Duration) {
    let mut curr_delay = DEFAULT_DELAY;

    loop {
        match av_rx.try_recv() {
            Ok(HeartbeatMessage::Slower) => {
                curr_delay /= 2;
                if curr_delay < min {
                    curr_delay = min;
                }
            }
            Ok(HeartbeatMessage::Faster) => {
                curr_delay *= 2;
                if curr_delay > max {
                    curr_delay = max;
                }
            }
            Ok(HeartbeatMessage::Die) | Err(TryRecvError::Disconnected) => {
                break;
            }
            Err(TryRecvError::Empty) => {}
        }

        std::thread::sleep(curr_delay);

        match hb_tx.send(()) {
            Ok(_) => {}
            Err(_) => {
                curr_delay *= 2;
            }
        }
    }
}
