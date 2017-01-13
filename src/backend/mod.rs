#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Cell;
use std::collections::VecDeque;
use std::iter::FromIterator;

type LineBuffer = VecDeque<char>;

trait EditableLine {
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
        for c in chars {
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

struct TextBuffer {
    lines: VecDeque<LineBuffer>,
}

impl TextBuffer {
    fn new() -> TextBuffer {
        let mut lines = VecDeque::new();
        lines.push_back(EditableLine::new());
        TextBuffer { lines: lines }
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

struct Position {
    row: usize,
    col: usize,
    range: usize,
}

enum ChangeType {
    Insert,
    Delete,
}

struct Change {
    pos: Vec<Position>,
    text: String,
    ctype: ChangeType,
}

struct Editor {
    buffer: TextBuffer,
    main_caret: Position,
    sub_caret: Vec<Position>,
    undo_pool: Rc<RefCell<Vec<Change>>>,
    redo_pool: Rc<RefCell<Vec<Change>>>,
    modified: bool,
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
        }
    }
    pub fn forward(&mut self) {
        self.main_caret.col += 1;
        let line = self.buffer.get(self.main_caret.row).expect("Caret out of bounds!");
        let len = line.len();
        if len < self.main_caret.col {
            if self.buffer.len() <= self.main_caret.row {
                self.main_caret.col = len;
            } else {
                self.main_caret.col = 0;
                self.main_caret.row += 1;
            }
        }
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
        editor.insert(String::from("Hello "));
        editor.insert(String::from("world!!"));
        assert_eq!(editor.get_all(), "Hello world!!");
    }
}