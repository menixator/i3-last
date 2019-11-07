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

use i3ipc::reply;

// The maximum number of windows in either stack. The state object can hold a maximum of
// 2*MAX_WINDOWS+1 windows in its memory.
const MAX_WINDOWS: usize = 15;

// Internal event enum. These can be triggered from the thread that watches for signals, or from
// the thread that watches i3 for any changes.
#[derive(Clone)]
pub enum Event {
    // When the SIGINT signal is received, this event is triggered.
    EXIT,

    // These three events are triggered when the user wants to move around.
    FORWARD,
    BACKWARD,
    LAST,

    // Following two events are triggered by the thread listening for events from i3. Both the
    // events have a Node type value attached to it, which is the window abstraction.
    FOCUSCHANGED(reply::Node),
    WINDOWCLOSED(reply::Node),
}

pub struct State {
    // The stack that holds the list of windows that the user has already visited.
    pub previous: Vec<i64>,

    // The stack that holds the list of windows that the user has decided to move back over.
    pub newer: Vec<i64>,

    // If i3-last is controlling i3, this i64 will have been populated with a window id that was
    // being focused. If the focus was changed this member will have an Option<64> bit integer that
    // refers to a window.
    pub ench_winid: Option<i64>,

    // The last enchant that the state issued. Used to jump to the last window. This can only be
    // `Event::FORWARD`, `Event::BACKWARD`, or None.
    pub last_enchant: Option<Event>,
    // Current window id
    pub current: Option<i64>,
}

impl State {
    pub fn new() -> Self {
        State {
            // Both the stacks get initialized as empty vectors.
            previous: Vec::new(),
            newer: Vec::new(),

            // Default values for the remaining properties.
            ench_winid: None,
            current: None,
            last_enchant: None,
        }
    }

    // A helper method that removes any elements longer than MAX_WINDOWS from a vector type.
    fn clamp(vec: &mut Vec<i64>) {
        if vec.len() > MAX_WINDOWS {
            vec.drain(0..(vec.len() - MAX_WINDOWS));
        }
    }

    // Helper to remove an item from a vector. The remove_item method is unstable at the moment.
    fn remove_from_vec(vec: &mut Vec<i64>, window_id: i64) -> Option<bool> {
        let found = vec.iter().position(|&id| id == window_id);
        match found {
            None => Some(false),
            Some(index) => {
                vec.remove(index);
                Some(true)
            }
        }
    }

    // Helper to remove a window id from all the stacks after it has been closed.
    pub fn purge(&mut self, id: i64) {
        State::remove_from_vec(&mut self.previous, id);
        State::remove_from_vec(&mut self.newer, id);
        match self.current {
            Some(current) if current == id => {
                self.current = None;
            }
            _ => {}
        }
    }

    // Public method to allow user to move forwards in the history.
    pub fn next(&mut self) -> Option<i64> {
        return self.seek(Event::FORWARD);
    }

    // Public method to allow user to move backwards in the history.
    pub fn prev(&mut self) -> Option<i64> {
        return self.seek(Event::BACKWARD);
    }

    // Adds a window to the history. This method is called whenever a new window is focused.
    pub fn add_window(&mut self, window_id: i64) {
        // Check if the state object issued any commands to i3
        if let Some(ench_winid) = self.ench_winid {
            // Check if the currently focused window is the window that we wanted to focus on
            if ench_winid == window_id {
                // If it is indeed that window, reset the ench_winid and return. The history will
                // not get modified.
                self.ench_winid = None;
                return;
            } else {
                // Assume a failure, and add the currently focused window to the history.
                self.ench_winid = None;
            }
        }

        // Move the window id in `current` - the window that was focused to the stack of windows
        // that we have already visited.
        if let Some(current) = self.current {
            // Make sure that there are no duplicates.
            State::remove_from_vec(&mut self.previous, current);
            self.previous.push(current);
            // Whenever a new window is focused by the user, while moving backwards, the history of the windows we have
            // moved backwards through will get reset.
            self.newer.clear();
            // Assure that the length is still under limits.
            State::clamp(&mut self.previous);
        }

        // Make sure that the newly focused window does not exist in either stack.
        State::remove_from_vec(&mut self.previous, window_id);
        State::remove_from_vec(&mut self.newer, window_id);

        // Treat any focus event as an enchant, therefore, Event::BACK will go back correctly.
        self.last_enchant = Some(Event::FORWARD);

        // Set the `current` window to the new window id.
        self.current = Some(window_id);
    }

    // A helper function that can respond to `BACKWARD` and `FORWARD` events.
    fn seek(&mut self, action: Event) -> Option<i64> {
        // The two vectors being operated upon.
        let remove_from: &mut Vec<i64>;
        let mut add_to: &mut Vec<i64>;

        match action {
            // If we are moving backwards, we will remove from previous and add to the newer stack.
            Event::BACKWARD => {
                remove_from = &mut self.previous;
                add_to = &mut self.newer;
            }

            // If we are moving forwards, we will remove from newer and add to the previous stack.
            Event::FORWARD => {
                remove_from = &mut self.newer;
                add_to = &mut self.previous;
            }
            _ => {
                panic!("unacceptable!");
            }
        }

        // If there is nothing to remove, return.
        if remove_from.len() == 0 {
            return None;
        }

        match remove_from.pop() {
            None => return None,
            Some(win_id) => {
                // The state will be enchanted till the next focus event. If the next window that
                // gets focused upon has the same window id as win_id, the focus event will have no
                // effect on the stacks. However, the state will become disenchanted.
                self.ench_winid = Some(win_id);

                // Clone and save the action we are doing so that we can reverse it.
                self.last_enchant = Some(action.clone());

                // Move the currently focused window's id to the add_to vector.
                if let Some(current) = self.current {
                    // Prevent any duplicates.
                    State::remove_from_vec(&mut add_to, current);
                    add_to.push(current);
                    // Clamping the length.
                    State::clamp(&mut add_to);
                }
                self.current = Some(win_id);
                return Some(win_id);
            }
        }
    }
}

// Tests if the length of the vector that holds the windows gets clamped correctly.
// When more than MAX_WINDOWS windows are added, the vector should remove older window ids.
#[test]
fn check_clamping() {
    let mut state = State::new();

    let max = 100;
    for i in 1..max {
        state.add_window(i);
    }

    assert_eq!(state.next(), None);
    for i in 0..MAX_WINDOWS {
        assert_eq!(state.prev(), Some(max - (i as i64) - 2));
    }
}
