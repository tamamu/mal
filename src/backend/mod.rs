#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Cell;
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::io::Read;

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
    pub range: usize,
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
    pub main_caret: Position,
    sub_caret: Vec<Position>,
    undo_pool: Rc<RefCell<Vec<Change>>>,
    redo_pool: Rc<RefCell<Vec<Change>>>,
    modified: bool,
    filename: String,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            buffer: TextBuffer::new(),
            main_caret: Position {
                row: 0,
                col: 0,
                range: 0,
            },
            sub_caret: Vec::new(),
            undo_pool: Rc::new(RefCell::new(Vec::new())),
            redo_pool: Rc::new(RefCell::new(Vec::new())),
            modified: false,
            filename: String::from("Untitled"),
        }
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
        let conc_line = self.buffer
                            .get_mut(self.main_caret.row)
                            .expect("Line out of bounds!")
                            .split_off(self.main_caret.col);
        self.buffer.new_line(self.main_caret.row + 1, conc_line);
        self.main_caret.row += 1;
        self.main_caret.col = 0;
    }
    pub fn backspace(&mut self) {
        if self.main_caret.col == 0 {
            if self.main_caret.row > 0 {
                let mut conc_line = self.buffer
                                        .get(self.main_caret.row)
                                        .expect("Line out of bounds!")
                                        .clone();
                {
                    let mut prev_line = self.buffer
                                            .get_mut(self.main_caret.row - 1)
                                            .expect("Line out of bounds!");
                    prev_line.append(&mut conc_line);
                }
                self.buffer.remove(self.main_caret.row);
                self.main_caret.row -= 1;
                self.main_caret.col = self.buffer
                                          .get(self.main_caret.row)
                                          .expect("Line out of bounds!")
                                          .len();
            }
        } else {
            let removed_char = self.buffer
                                   .get_mut(self.main_caret.row)
                                   .expect("Caret out of bounds!")
                                   .remove(self.main_caret.col - 1);
            match removed_char {
                Some(c) => {
                    let backward_length = if c.len_utf8() > 1 {
                        2
                    } else {
                        1
                    };
                    self.main_caret.col -= backward_length;
                }
                None => {}
            }
        }
    }
    pub fn move_left(&mut self) {
        if self.main_caret.col == 0 {
            if self.main_caret.row > 0 {
                self.main_caret.row -= 1;
                self.main_caret.col = self.buffer
                                          .get(self.main_caret.row)
                                          .expect("Caret out of bounds!")
                                          .len();
            }
        } else {
            let backward_length = match self.buffer
                                            .get(self.main_caret.row)
                                            .expect("Caret out of bounds!")
                                            .get(self.main_caret.col - 1) {
                Some(c) => {
                    if c.len_utf8() > 1 {
                        2
                    } else {
                        1
                    }
                }
                None => 1,
            };
            self.main_caret.col -= backward_length;
        }
    }
    pub fn move_right(&mut self) {
        let forward_length = match self.buffer
                                       .get(self.main_caret.row)
                                       .expect("Caret out of bounds!")
                                       .get(self.main_caret.col) {
            Some(c) => {
                if c.len_utf8() > 1 {
                    2
                } else {
                    1
                }
            }
            None => 1,
        };
        self.main_caret.col += forward_length;
        let line = self.buffer.get(self.main_caret.row).expect("Caret out of bounds!");
        let len = line.len();
        if len < self.main_caret.col {
            if self.buffer.len() - 1 <= self.main_caret.row {
                self.main_caret.col = len;
            } else {
                self.main_caret.col = 0;
                self.main_caret.row += 1;
            }
        }
    }
    pub fn move_top(&mut self) {
        self.main_caret.row = 1;
        self.main_caret.col = 1;
    }
    pub fn move_up(&mut self) {
        if self.main_caret.row > 0 {
            self.main_caret.row -= 1;
            let len = self.buffer.get(self.main_caret.row).expect("Line out of bounds!").len();
            if len < self.main_caret.col {
                self.main_caret.col = len;
            }
        } else {
            self.main_caret.col = 0;
        }
    }
    pub fn move_down(&mut self) {
        if self.main_caret.row < self.buffer.len() - 1 {
            self.main_caret.row += 1;
            let len = self.buffer.get(self.main_caret.row).expect("Line out of bounds!").len();
            if len < self.main_caret.col {
                self.main_caret.col = len;
            }
        } else {
            self.main_caret.col = self.buffer
                                      .get(self.main_caret.row)
                                      .expect("Caret out of bounds!")
                                      .len();
        }
    }
    pub fn insert_char(&mut self, c: char) {
        let line = &mut self.buffer.get_mut(self.main_caret.row).expect("Caret out of bounds!");
        line.insert(self.main_caret.col, c);
        let forward_length = if c.len_utf8() > 1 {
            2
        } else {
            1
        };
        self.main_caret.col += 1;
    }
    pub fn insert(&mut self, text: String) {
        let line = &mut self.buffer.get_mut(self.main_caret.row).expect("Caret out of bounds!");
        if line.len() == self.main_caret.col {
            self.main_caret.col += line.input_back(text);
        } else if self.main_caret.col == 0 {
            self.main_caret.col += line.input_front(text);
        } else {
            self.main_caret.col += line.input_at(&self.main_caret.col, text);
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