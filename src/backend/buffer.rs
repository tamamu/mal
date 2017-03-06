
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

pub struct Line<T> {
    head: Option<Box<Node<T>>>,
}

pub struct Buffer<T> {
    lines: Box<Node<Line<T>>>
}

use termion::color::Color;
pub struct Letter {
    fg: usize,
		bg: usize,
		bold: bool,
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
