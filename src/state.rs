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

use i3ipc::reply;
use i3ipc::I3Connection;

const MAX_WINDOWS: usize = 15;

#[derive(Clone)]
pub enum Event {
    EXIT,
    FORWARD,
    BACKWARD,
    LAST,
    FOCUSCHANGED(reply::Node),
    WINDOWCLOSED(reply::Node),
}

pub struct State {
    pub previous: Vec<i64>,
    pub newer: Vec<i64>,
    pub enchanted: bool,
    pub ench_winid: i64,
    pub last_enchant: Option<Event>,
    pub connection: I3Connection,
    // Current window id
    pub current: i64,
}

impl State {
    pub fn new() -> Self {
        State {
            previous: Vec::new(),
            newer: Vec::new(),
            enchanted: false,
            ench_winid: -1,

            current: -1,
            connection: I3Connection::connect().unwrap(),
            last_enchant: None,
        }
    }

    fn clamp(vec: &mut Vec<i64>) {
        if vec.len() > MAX_WINDOWS {
            vec.drain(MAX_WINDOWS..vec.len());
        }
    }

    fn remove_from_vec(vec: &mut Vec<i64>, window_id: i64) -> Option<bool> {
        let found = vec.iter().position(|&id| id == window_id);
        match found {
            None => Some(false),
            Some(_) => {
                vec.remove(found.unwrap());
                Some(true)
            }
        }
    }

    pub fn purge(&mut self, id: i64) {
        State::remove_from_vec(&mut self.previous, id);
        State::remove_from_vec(&mut self.newer, id);
        if self.current == id {
            self.current = -1;
        }
    }

    pub fn next(&mut self) {
        self.seek(Event::FORWARD);
    }

    pub fn prev(&mut self) {
        self.seek(Event::BACKWARD);
    }

    pub fn add_window(&mut self, window_id: i64) {
        if self.enchanted {
            self.enchanted = false;

            // Check if the currently focused window is the ench_winid
            if self.ench_winid == window_id {
                // ignore it.
                self.ench_winid = -1;
                return;
            } else {
                self.ench_winid = -1;
            }
        }

        // the user moved
        if self.current != -1 {
            State::remove_from_vec(&mut self.previous, self.current);
            self.previous.push(self.current);
            self.newer.clear();
            State::clamp(&mut self.previous);
        }
        State::remove_from_vec(&mut self.previous, window_id);
        State::remove_from_vec(&mut self.newer, window_id);

        State::clamp(&mut self.newer);

        if self.current != -1 {
            State::clamp(&mut self.previous);
        }

        self.current = window_id;
    }

    fn seek(&mut self, action: Event) {
        let remove_from: &mut Vec<i64>;
        let mut add_to: &mut Vec<i64>;

        match action {
            Event::BACKWARD => {
                remove_from = &mut self.previous;
                add_to = &mut self.newer;
            }
            Event::FORWARD => {
                remove_from = &mut self.newer;
                add_to = &mut self.previous;
            }
            _ => {
                panic!("unacceptable!");
            }
        }

        if remove_from.len() == 0 {
            return;
        }

        match remove_from.pop() {
            None => return,
            Some(win_id) => {
                self.enchanted = true;
                self.ench_winid = win_id;
                self.last_enchant = Some(action.clone());

                if self.current != -1 {
                    State::remove_from_vec(&mut add_to, self.current);
                    add_to.push(self.current);
                    State::clamp(&mut add_to);
                }
                self.current = win_id;

                self.connection
                    .run_command(&format!("[con_id={}] focus", win_id))
                    .ok();
            }
        }
    }
}
