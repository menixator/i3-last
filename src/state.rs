use i3ipc::reply;
use i3ipc::I3Connection;

const MAX_WINDOWS: usize = 15;

#[derive(Copy, Clone)]
pub enum Action {
    EXIT,
    FORWARD,
    BACKWARD,
    LAST,
    FOCUSCHANGED,
    WINDOWCLOSED,
}

pub struct Event {
    pub variant: Action,
    pub container: Option<reply::Node>,
}
pub struct State {
    pub previous: Vec<i64>,
    pub newer: Vec<i64>,
    pub enchanted: bool,
    pub ench_winid: i64,
    pub last_enchant: Option<Action>,
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
        self.seek(Action::FORWARD);
    }

    pub fn prev(&mut self) {
        self.seek(Action::BACKWARD);
    }

    pub fn add_window(&mut self, window_id: i64) {
        if self.enchanted {
            self.enchanted = false;

            // Check if the currently focused window
            // is the ench_winid
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

    fn seek(&mut self, action: Action) {
        let remove_from: &mut Vec<i64>;
        let mut add_to: &mut Vec<i64>;

        match action {
            Action::BACKWARD => {
                remove_from = &mut self.previous;
                add_to = &mut self.newer;
            }
            Action::FORWARD => {
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
