#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Cell;
use std::collections::VecDeque;

struct LineBuffer = VecDeque<Char>;

impl LineBuffer {
  pub fn new() -> LineBuffer {
    VecDeque::new()
  }
  pub fn input_front(&mut self, value: String) -> usize {
    let mut iter = value.chars().iter();
    let mut len: usize = 0;
    for c in iter {
      self.push_front(c);
      len += 1;
    }
    len
  }
  pub fn input_at(&mut self, index: &usize, value: String) -> usize {
    let mut iter = value.chars().iter().enumerate();
    let mut len: usize = 0;
    for stride, c in iter {
      self.insert(index+stride, c);
      len += 1;
    }
    len
  }
  
  pub fn input_back(&mut self, value: String) -> usize {
    let mut iter = value.chars().iter();
    let mut len: usize = 0;
    for c in iter {
      self.push_back(c);
      len += 1;
    }
    len
  }
}

struct TextBuffer = VecDeque<LineBuffer>;

impl TextBuffer {
  pub fn new() -> TextBuffer {
    VecDeque::new()
  }
}

struct Position {
  row: usize,
  col: usize,
  range: usize
}

enum ChangeType {
  Insert,
  Delete
}

struct Change {
  pos: Vec<Position>,
  text: String,
  type: ChangeType,
}

struct Editor {
  buffer: TextBuffer,
  main_caret: Position,
  sub_caret: Vec<Position>,
  undo_pool: Rc<RefCell<Vec<Change>>>,
  redo_pool: Rc<RefCell<Vec<Change>>>,
  modified: bool
}

impl Editor {
  pub fn new() -> Editor {
    Editor {
      buffer: TextBuffer::new(),
      main_caret: Position{row: 0, col: 0, range: 0},
      sub_caret: Vec::new(),
      undo_pool: Rc::new(RefCell::new(Vec::new())),
      redo_pool: Rc::new(RefCell::new(Vec::new())),
      modified: false
    }
  }
  pub fn insert(&mut self, text: String) {
    let line = &mut self.buffer[self.main_caret.row];
    if line.len() == self.main_caret.col {
      self.main_caret.col += line.input_back(text);
    } else if self.main_caret.col == 0 {
      self.main_caret.col += line.input_front(text);
    } else {
      self.main_caret.col += line.input_at(&self.main_caret.col, text);
    }
  } 
}