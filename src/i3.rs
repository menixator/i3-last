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

// Renamed as i3Event to prevent clash with internal
// Event type
use i3ipc::{
    event::{inner::WindowChange, Event as i3Event, WindowEventInfo},
    I3EventListener, Subscription,
};
use std::process::exit;
use std::sync::mpsc::Sender;
use std::thread;

// Import internal types
use super::state::{Action, Event};

pub fn spawn_i3listener(i3tx: Sender<Event>) {
    // Connect to i3
    // or die if i3 can't be reached.
    let mut listener = match I3EventListener::connect() {
        Ok(listener) => listener,
        Err(_) => {
            eprintln!("failed to connect to i3. is i3 running?");
            exit(1);
        }
    };
    // Spawn a thread that runs forever
    thread::spawn(move || {
        // Subscribe to only window events.
        // We aren't interested in anything else
        let subs = [Subscription::Window];

        match listener.subscribe(&subs) {
            Ok(_) => {}
            Err(err) => {
                panic!("failed to subscribe to i3 events due to err: {:?}", err);
            }
        }
        // Create an iterator for events

        for event in listener.listen() {
            // Matches events from i3 and maps them as internal Actions
            // and Events(which can be sent over the i3tx sender half.
            match event {
                Ok(i3Event::WindowEvent(WindowEventInfo {
                    change: WindowChange::Close,
                    container,
                    ..
                })) => {
                    i3tx.send(Event {
                        variant: Action::WINDOWCLOSED,
                        container: Some(container),
                    })
                    .unwrap();
                }
                Ok(i3Event::WindowEvent(WindowEventInfo {
                    change: WindowChange::Focus,
                    container,
                    ..
                })) => {
                    i3tx.send(Event {
                        variant: Action::FOCUSCHANGED,
                        container: Some(container),
                    })
                    .unwrap();
                }
                Err(err) => {
                    eprintln!("unknown error within subscription: {}", err);
                }
                _ => (),
            }
        }
    });
}
