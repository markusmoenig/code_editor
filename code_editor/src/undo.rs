pub struct Undo {
    pub undo_data           : String,
    pub redo_data           : String,
    pub redo_pos            : (usize, usize),

    pub time_stamp          : u128,
}

pub struct UndoStack {
    pub stack               : Vec<Undo>,

    pub index               : isize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            stack           : vec![],
            index           : -1,
        }
    }

    pub fn has_undo(&self) -> bool {
        self.index >= 0
    }

    pub fn has_redo(&self) -> bool {
        if self.index >= -1 && self.index < self.stack.len() as isize - 1 {
            return true;
        }
        false
    }

    pub fn undo(&mut self) -> String {
        let rc = self.stack[self.index as usize].undo_data.clone();
        self.index -= 1;
        rc
    }

    pub fn redo(&mut self) -> (String, (usize, usize)) {
        self.index += 1;
        let rc = (self.stack[self.index as usize].redo_data.clone(), self.stack[self.index as usize].redo_pos.clone());
        rc
    }

    pub fn add(&mut self, undo: String, redo: String, redo_pos: (usize, usize)) {

        if self.index >= 0 {

            let time = self.get_time();

            // If the last item is less than 2s old, replace it
            if time < self.stack[(self.index) as usize].time_stamp + 2000 {

                self.stack[(self.index) as usize].redo_data = redo;
                self.stack[(self.index) as usize].redo_pos = redo_pos;
                self.stack[(self.index) as usize].time_stamp = time;

                return;
            }
        }

        let to_remove = self.stack.len() as isize - self.index - 1;
        for _i in 0..to_remove {
            self.stack.pop();
        }

        self.stack.push(Undo {
            undo_data   : undo,
            redo_data   : redo,
            redo_pos    : redo_pos,
            time_stamp  : self.get_time(),
        });

        self.index += 1;
    }

    fn get_time(&self) -> u128 {
        let stop = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");
            stop.as_millis()
    }
}