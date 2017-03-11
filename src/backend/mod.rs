#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

use std::ops::{AddAssign, SubAssign};
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Cell;
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::io::Read;

// mod buffer;

pub type LineBuffer = VecDeque<char>;

pub trait EditableLine {
    fn new() -> Self;
    fn input_front(&mut self, text: String) -> usize;
    fn input_at(&mut self, index: &usize, text: String) -> usize;
    fn input_back(&mut self, text: String) -> usize;
    fn extract(&self) -> String;
}

impl EditableLine for LineBuffer {
    fn new() -> LineBuffer {
        VecDeque::new()
    }
    fn input_front(&mut self, text: String) -> usize {
        let chars = text.chars();
        let mut len: usize = 0;
        for c in chars.rev() {
            self.push_front(c);
            len += 1;
        }
        len
    }
    fn input_at(&mut self, index: &usize, text: String) -> usize {
        let chars = text.chars().enumerate();
        let mut len: usize = 0;
        for (stride, c) in chars {
            self.insert(index + stride, c);
            len += 1;
        }
        len
    }

    fn input_back(&mut self, text: String) -> usize {
        let chars = text.chars();
        let mut len: usize = 0;
        for c in chars {
            self.push_back(c);
            len += 1;
        }
        len
    }
    fn extract(&self) -> String {
        String::from_iter(self.into_iter().cloned())
    }
}

pub struct TextBuffer {
    lines: VecDeque<LineBuffer>,
}

impl TextBuffer {
    fn new() -> TextBuffer {
        let mut lines = VecDeque::new();
        lines.push_back(EditableLine::new());
        TextBuffer { lines: lines }
    }
    fn new_line(&mut self, index: usize, line: LineBuffer) {
        self.lines.insert(index, line);
    }
    fn remove(&mut self, index: usize) -> Option<LineBuffer> {
        self.lines.remove(index)
    }
    fn get(&self, index: usize) -> Option<&LineBuffer> {
        self.lines.get(index)
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut LineBuffer> {
        self.lines.get_mut(index)
    }
    fn back(&self) -> Option<&LineBuffer> {
        self.lines.back()
    }
    fn len(&self) -> usize {
        self.lines.len()
    }
    fn extract(&self) -> String {
        let mut view = String::new();
        let mut iter = self.lines.iter();
        let first_line = iter.next();
        match first_line {
            None => {}
            Some(text) => {
                view.push_str(&text.extract());
                for line in iter {
                    view.push('\n');
                    view.push_str(&line.extract());
                }
            }
        }
        view
    }
}

pub struct Position {
    pub row: usize,
    pub col: usize,
    pub range: isize,
}

enum EditType {
    Normal,
    Select,
}

enum ChangeType {
    Insert,
    Delete,
}

pub struct Change {
    pos: Vec<Position>,
    text: String,
    ctype: ChangeType,
}

pub struct Editor {
    buffer: TextBuffer,
    pub carets: Vec<Position>,
    undo_pool: Rc<RefCell<Vec<Change>>>,
    redo_pool: Rc<RefCell<Vec<Change>>>,
    modified: bool,
    filename: String,
    edit_type: EditType,
}

impl Editor {
    pub fn new() -> Editor {
        let mut carets = Vec::new();
        carets.push(Position {
            row: 0,
            col: 0,
            range: 0,
        });
        Editor {
            buffer: TextBuffer::new(),
            carets: carets,
            undo_pool: Rc::new(RefCell::new(Vec::new())),
            redo_pool: Rc::new(RefCell::new(Vec::new())),
            modified: false,
            filename: String::from("Untitled"),
            edit_type: EditType::Normal,
        }
    }
    pub fn mode_select(&mut self) {
        self.edit_type = EditType::Select;
    }
    pub fn mode_normal(&mut self) {
        self.edit_type = EditType::Normal;
    }
    pub fn read_file<P: AsRef<Path>>(&mut self, path: P) {
        let mut file = match File::open(path.as_ref()) {
            Err(why) => {
                panic!("Couldn't open {}: {}",
                       path.as_ref().display(),
                       why.description())
            }
            Ok(file) => file,
        };
        let mut tmp = String::new();
        match file.read_to_string(&mut tmp) {
            Err(why) => {
                panic!("Couldn't read {}: {}",
                       path.as_ref().display(),
                       why.description())
            }
            Ok(_) => {}
        }
        let mut lines = tmp.lines();
        self.buffer.lines = VecDeque::new();
        for line in lines {
            let lb: LineBuffer = line.chars().collect();
            self.buffer.lines.push_back(lb);
        }
    }
    pub fn insert_line(&mut self) {
        for caret in self.carets.iter_mut() {
            let conc_line = self.buffer
                                .get_mut(caret.row)
                                .expect("Line out of bounds!")
                                .split_off(caret.col);
            self.buffer.new_line(caret.row + 1, conc_line);
            caret.row += 1;
            caret.col = 0;
        }
    }
    pub fn backspace(&mut self) {
        let main_caret = self.carets.get_mut(0).expect("Caret not found!");
        if main_caret.col == 0 {
            if main_caret.row > 0 {
                let mut conc_line = self.buffer
                                        .get(main_caret.row)
                                        .expect("Line out of bounds!")
                                        .clone();
                {
                    let mut prev_line = self.buffer
                                            .get_mut(main_caret.row - 1)
                                            .expect("Line out of bounds!");
                    prev_line.append(&mut conc_line);
                }
                self.buffer.remove(main_caret.row);
                main_caret.row -= 1;
                main_caret.col = self.buffer
                                     .get(main_caret.row)
                                     .expect("Line out of bounds!")
                                     .len();
            }
        } else {
            let removed_char = self.buffer
                                   .get_mut(main_caret.row)
                                   .expect("Caret out of bounds!")
                                   .remove(main_caret.col - 1);
            match removed_char {
                Some(c) => {
                    main_caret.col -= 1;
                }
                None => {}
            }
        }
    }
    pub fn move_left(&mut self) {
        match self.edit_type {
            EditType::Normal => {
                for caret in self.carets.iter_mut() {
                    if caret.col == 0 {
                        if caret.row > 0 {
                            caret.row -= 1;
                            caret.col = self.buffer
                                            .get(caret.row)
                                            .expect("Caret out of bounds!")
                                            .len();
                        }
                    } else {
                        caret.col -= 1;
                    }
                }
            }
            EditType::Select => {
                for caret in self.carets.iter_mut() {
                    caret.range -= 1;
                    let ranged_col = caret.range + caret.col as isize;
                    if ranged_col < 0 {
                        panic!("select to before line.");
                    }
                }
            }
        }

    }
    pub fn move_right(&mut self) {
        match self.edit_type {
            EditType::Normal => {
                for caret in self.carets.iter_mut() {
                    caret.col += 1;
                    let line = self.buffer.get(caret.row).expect("Caret out of bounds!");
                    let len = line.len();
                    if len < caret.col {
                        if self.buffer.len() - 1 <= caret.row {
                            caret.col = len;
                        } else {
                            caret.col = 0;
                            caret.row += 1;
                        }
                    }
                }
            }
            EditType::Select => {
                let row_max = self.len() - 1;
                let col_row_max = self.get(row_max).expect("Line out of bounds!").len();
                for caret in self.carets.iter_mut() {
                    caret.range += 1;
                    let ranged_col = caret.range + caret.col as isize;
                    if caret.row == row_max && ranged_col > col_row_max as isize {
                        caret.range = (col_row_max - caret.col) as isize;
                    }
                }
            }
        }

    }
    pub fn move_top(&mut self) {
        for caret in self.carets.iter_mut() {
            caret.row = 0;
            caret.col = 0;
        }
    }
    pub fn move_end(&mut self) {
        let row = self.buffer.len() - 1;
        let col = self.buffer.back().expect("Buffer is empty!").len();
        for caret in self.carets.iter_mut() {
            caret.row = row;
            caret.col = col;
        }
    }
    pub fn move_pageup(&mut self, row: usize) {
        for caret in self.carets.iter_mut() {
            if caret.row < row {
                caret.row = 0;
                caret.col = 0;
            } else {
                caret.row -= row;
                let len = self.buffer.get(caret.row).expect("Line out of bounds!").len();
                if len < caret.col {
                    caret.col = len;
                }
            }
        }
    }
    pub fn move_pagedown(&mut self, row: usize) {
        let row_max = self.buffer.len() - 1;
        let col_row_max = self.buffer.get(row_max).expect("Line out of bounds!").len();
        for caret in self.carets.iter_mut() {
            if caret.row + row > row_max {
                caret.row = row_max;
                caret.col = col_row_max;
            } else {
                caret.row += row;
                let len = self.buffer.get(caret.row).expect("Line out of bounds!").len();
                if len < caret.col {
                    caret.col = len;
                }
            }
        }
    }
    pub fn move_up(&mut self) {
        for caret in self.carets.iter_mut() {
            if caret.row > 0 {
                caret.row -= 1;
                let len = self.buffer.get(caret.row).expect("Line out of bounds!").len();
                if len < caret.col {
                    caret.col = len;
                }
            } else {
                caret.col = 0;
            }
        }
    }
    pub fn move_down(&mut self) {
        for caret in self.carets.iter_mut() {
            if caret.row < self.buffer.len() - 1 {
                caret.row += 1;
                let len = self.buffer.get(caret.row).expect("Line out of bounds!").len();
                if len < caret.col {
                    caret.col = len;
                }
            } else {
                caret.col = self.buffer
                                .get(caret.row)
                                .expect("Caret out of bounds!")
                                .len();
            }
        }

    }
    pub fn insert_char(&mut self, c: char) {
        let main_caret = self.carets.get_mut(0).expect("Caret not found!");
        let line = &mut self.buffer.get_mut(main_caret.row).expect("Caret out of bounds!");
        line.insert(main_caret.col, c);
        main_caret.col += 1;
    }
    pub fn insert(&mut self, text: String) {
        let main_caret = self.carets.get_mut(0).expect("Caret not found!");
        let line = &mut self.buffer.get_mut(main_caret.row).expect("Caret out of bounds!");
        if line.len() == main_caret.col {
            main_caret.col += line.input_back(text);
        } else if main_caret.col == 0 {
            main_caret.col += line.input_front(text);
        } else {
            main_caret.col += line.input_at(&main_caret.col, text);
        }
    }
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    pub fn get(&self, index: usize) -> Option<&LineBuffer> {
        self.buffer.get(index)
    }
    pub fn get_all(&self) -> String {
        self.buffer.extract()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_test1() {
        let mut editor = Editor::new();
        editor.insert(String::from("l"));
        editor.move_left();
        editor.insert(String::from("He"));
        editor.move_right();
        editor.insert(String::from("lo world!!"));
        assert_eq!(editor.get_all(), "Hello world!!");
    }
}
