// Multi-producer, single-consumer FIFO queue communication primitives for
// communication between the threads
use std::sync::mpsc::{channel as mkchannel, Receiver, Sender};

// The program's state is in a single structure
mod state;
// Import required components into the global scope
// so that it can be accessed without the namespace
use state::{Action, Event, State};

// The program's i3 interface.
// While I call it an interface, it only exports a single
// function that can take a Sender<Event> half of a channel
// and runs everything in isolation
mod i3;

// Signal handler thread exists here. Same as the i3 interface.
mod signals;

fn main() {
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
            Ok(Event {
                variant: Action::LAST,
                ..
            }) => match state.last_enchant {
                None => {
                    continue;
                }
                // If the last action was a backward action
                // do the opposite.
                Some(Action::BACKWARD) => {
                    state.next();
                }
                // If the last action was a forward action
                // do the opposite.
                Some(Action::FORWARD) => {
                    state.prev();
                }
                _ => {
                    // An almost impossible case.
                    // If this occurs, then there probably is an error
                    panic!("disallowed action set to last_enchant");
                }
            },
            // This action signifies when a window is closed.
            Ok(Event {
                variant: Action::WINDOWCLOSED,
                container,
            }) => {
                state.purge(container.unwrap().id);
            }
            // This action gets triggered when the user wants to go
            // backwards.
            Ok(Event {
                variant: Action::BACKWARD,
                ..
            }) => {
                // State contains the implementation
                state.prev();
            }
            // Same as above but for forward.
            Ok(Event {
                variant: Action::FORWARD,
                ..
            }) => {
                state.next();
            }

            // Updates the data in state when the focus changes
            Ok(Event {
                variant: Action::FOCUSCHANGED,
                container,
            }) => {
                state.add_window(container.unwrap().id);
            }

            // Exits the application
            Ok(Event {
                variant: Action::EXIT,
                ..
            }) => {
                break;
            }

            Err(err) => {
                panic!("a recieve error occured: {:?}", err);
            }
        }
    }
}
