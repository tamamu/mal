
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
    fn insert_back(&mut self, data: T) {
		    let node = Node{next: self.next,
				                Rawlink:some(&mut self),
												value: data};
				self.next = Some(Box::new(node));
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_test() {
        let mut foo = BLinkedList {
            prev: None,
            data: 5,
            next: None,
        };
        println!("{:?}", foo.insert_front(3));
    }
}
