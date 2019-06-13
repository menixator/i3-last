// i3-last - a helper program for i3 to switch between windows
// Copyright (C) 2019  Ahmed Miljau<ahmed.miljau@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
// Multi-producer, single-consumer FIFO queue communication primitives for
// communication between the threads

use signal_hook::{iterator::Signals, SIGINT};
use std::sync::mpsc::Sender;
use std::thread;

// Import the internal types
use super::state::Event;

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
                    sigtx.send(Event::FORWARD).unwrap();
                }
                SIG_BACKWARD => {
                    sigtx.send(Event::BACKWARD).unwrap();
                }
                SIG_LAST => {
                    sigtx.send(Event::LAST).unwrap();
                }

                SIGINT => {
                    sigtx.send(Event::EXIT).unwrap();
                    break;
                }
                _ => {}
            }
        }
    });
}
