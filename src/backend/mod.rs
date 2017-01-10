#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

use std::rc::Rc;
use std::cell::RefCell;
use std::cell::Cell;

struct LineBuffer = String;

struct TextBuffer = Vec<LineBuffer>;

struct Range<T> {
  from: T,
  to: T,
}

struct Position {
  row: usize,
  col: usize,
  range: Range<usize>,
}

enum ChangeType {
  Insert,
  Delete,
  Cut,
}

struct Change {
  pos: Position,
  text: String,
  type: ChangeType,
}

struct Editor {
  buffer: TextBuffer,
  cursor: Position,
  undo_pool: Rc<RefCell<Vec<Change>>>,
  redo_pool: Rc<RefCell<Vec<Change>>>,
  modified: bool
}