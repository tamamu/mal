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
}

struct TextBuffer = Vec<LineBuffer>;

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
      buffer: Vec::new(),
      main_caret: Position{row: 0, col: 0, range: 0},
      sub_caret: Vec::new(),
      undo_pool: Rc::new(RefCell::new(Vec::new())),
      redo_pool: Rc::new(RefCell::new(Vec::new())),
      modified: false
    }
  }
  pub fn insert(&mut self, text: String) {
    self.
  } 
}