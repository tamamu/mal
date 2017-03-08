
use std::iter::FromIterator;

use std::mem;
use std::ptr;

#[derive(Copy)]
struct Rawlink<T> {
    p: *mut T,
}

impl<T> Rawlink<T> {
    /// Like Option::None for Rawlink
    fn none() -> Rawlink<T> {
        Rawlink { p: ptr::null_mut() }
    }

    /// Like Option::Some for Rawlink
    fn some(n: &mut T) -> Rawlink<T> {
        Rawlink { p: n as *mut T }
    }

    /// Convert the `Rawlink` into an Option value
    fn resolve<'a>(&self) -> Option<&'a T> {
        unsafe { self.p.as_ref() }
    }

    /// Convert the `Rawlink` into an Option value
    fn resolve_mut<'a>(&mut self) -> Option<&'a mut T> {
        unsafe { self.p.as_mut() }
    }

    /// Return the `Rawlink` and replace with `Rawlink::none()`
    fn take(&mut self) -> Rawlink<T> {
        mem::replace(self, Rawlink::none())
    }
}

impl<T> Clone for Rawlink<T> {
    fn clone(&self) -> Self {
        Rawlink { p: self.p }
    }
}

pub struct Node<T> {
    next: Option<Box<Node<T>>>,
    prev: Rawlink<Node<T>>,
    value: T,
}

impl<T> Node<T> {
    fn new(v: T) -> Node<T> {
		    Node{value: v, next: None, prev: Rawlink::none()}
		}
    fn insert_back(&mut self, v: T) {
		    let node = Node{next: self.next,
				                prev: Rawlink:some(&mut self),
												value: v};
				self.next = Some(Box::new(node));
		}
		fn insert_front(&mut self, v: T) {
		    let node = Node{next: Some(Box::new(self)),
				                prev: self.prev,
												value: v};
			  self.prev = Rawlink::some(&mut self);
		}
}

pub struct Line<T> {
    head: Option<Box<Node<T>>>,
}

pub struct Buffer<T> {
    lines: Box<Node<Line<T>>>
}

pub enum Color {
    Black,
		Red,
		Green,
		Yellow,
		Blue,
		Magenta,
		Cyan,
		White,
		Byte(u16),
}

pub struct Style {
    normal_color: bool,
		bold: bool,
		underline: bool,
		reverse: bool,
}

pub struct Letter {
    fg: Color,
		bg: Color,
		style: Style,
		glyph: char,
		width: usize,
}

pub type EditorBuffer = Buffer<Letter>;

impl EditorBuffer {
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_test() {
        let mut foo = EditorBuffer {
				    prev: None,
            data: 5,
            next: None,
        };
        println!("{:?}", foo.insert_front(3));
    }
}
