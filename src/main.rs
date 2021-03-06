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
use i3ipc::I3Connection;
use std::sync::mpsc::{channel as mkchannel, Receiver, Sender};
// The program's state is in a single structure
mod state;
// Import required components into the global scope
// so that it can be accessed without the namespace
use state::{Event, State};
use std::process::exit;

// The program's i3 interface.
// While I call it an interface, it only exports a single
// function that can take a Sender<Event> half of a channel
// and runs everything in isolation
mod i3;

// Signal handler thread exists here. Same as the i3 interface.
mod signals;

fn main() {
    // Create a connection to i3
    let mut connection: I3Connection = match I3Connection::connect() {
        Ok(connection) => connection,
        Err(err) => {
            eprintln!("failed to connect to i3 due to err: {:?}", err);
            exit(1);
        }
    };

    // Initialize the channels
    let (tx, rx): (Sender<Event>, Receiver<Event>) = mkchannel();

    // Spawn the threads
    // i3 to listen for i3 events
    i3::spawn_i3listener(tx.clone());
    // signals to listen for signal events
    signals::spawn_siglistener(tx.clone());

    // The program's state
    let mut state = State::new();

    // This is an infinite loop. The program will run forever
    // as a daemon till it recieves SIGINT
    // If it recieves an unknown signal, it panics and dies
    loop {
        // The messages recieved on rx are safe. Hopefully. So a naked
        // unwrap is not an issue
        match rx.recv() {
            // This is send on the channel when the user
            // requests to jump back to the last window.
            // last_enchant will contain the last action taken
            // by the program
            Ok(Event::LAST) => match state.last_enchant {
                None => {
                    continue;
                }
                // If the last action was a backward action
                // do the opposite.
                Some(Event::BACKWARD) => {
                    focus(&mut connection, state.next());
                }
                // If the last action was a forward action
                // do the opposite.
                Some(Event::FORWARD) => {
                    focus(&mut connection, state.prev());
                }
                _ => {
                    // An almost impossible case.
                    // If this occurs, then there probably is an error
                    panic!("disallowed action set to last_enchant");
                }
            },
            // This action signifies when a window is closed.
            Ok(Event::WINDOWCLOSED(container)) => {
                state.purge(container.id);
            }
            // This action gets triggered when the user wants to go
            // backwards.
            Ok(Event::BACKWARD) => {
                // State contains the implementation
                focus(&mut connection, state.prev());
            }
            // Same as above but for forward.
            Ok(Event::FORWARD) => {
                focus(&mut connection, state.next());
            }

            // Updates the data in state when the focus changes
            Ok(Event::FOCUSCHANGED(container)) => {
                state.add_window(container.id);
            }

            // Exits the application
            Ok(Event::EXIT) => {
                break;
            }

            Err(err) => {
                panic!("a recieve error occured: {:?}", err);
            }
        }
    }
}

// Helper to focus on a window.
fn focus(connection: &mut I3Connection, result: Option<i64>) {
    if let Some(win_id) = result {
        connection
            .run_command(&format!("[con_id={}] focus", win_id))
            .ok();
    }
}
