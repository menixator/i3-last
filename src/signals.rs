use signal_hook::{iterator::Signals, SIGINT};
use std::sync::mpsc::Sender;
use std::thread;

// Import the internal types
use super::state::{Action, Event};

use std::process::exit;
// SIGRTMIN isn't actually 36. Conventionally SIGRTMIN has 32 but, the glibc POSIX threads
// implementation internally uses two signals so SIGRTMIN can get adjusted to 36. Which is
// why it is not safe to hardcode this. I have taken the easier approach and left all the
// signals before 36 alone to account for this if the need ever arises(unklikely).
pub const SIGRTMIN_SAFE: i32 = 36;
// Three signals are used.
pub const SIG_FORWARD: i32 = SIGRTMIN_SAFE + 0;
pub const SIG_BACKWARD: i32 = SIGRTMIN_SAFE + 1;
pub const SIG_LAST: i32 = SIGRTMIN_SAFE + 2;

pub fn spawn_siglistener(sigtx: Sender<Event>) {
    let signals = match Signals::new(&[SIGINT, SIG_FORWARD, SIG_BACKWARD, SIG_LAST]) {
        Ok(signals) => signals,
        Err(err) => {
            eprintln!("failed to catch signals due to err: {:?}", err);
            exit(1);
        }
    };
    thread::spawn(move || {
        // This is a simple loop that runs forever
        // and catches any signals.
        for sig in signals.forever() {
            match sig {
                SIG_FORWARD => {
                    sigtx
                        .send(Event {
                            variant: Action::FORWARD,
                            container: None,
                        })
                        .unwrap();
                }
                SIG_BACKWARD => {
                    sigtx
                        .send(Event {
                            variant: Action::BACKWARD,
                            container: None,
                        })
                        .unwrap();
                }
                SIG_LAST => {
                    sigtx
                        .send(Event {
                            variant: Action::LAST,
                            container: None,
                        })
                        .unwrap();
                }

                SIGINT => {
                    sigtx
                        .send(Event {
                            variant: Action::EXIT,
                            container: None,
                        })
                        .unwrap();
                    break;
                }
                _ => {}
            }
        }
    });
}
