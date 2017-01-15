#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

extern crate x11;
extern crate gl;
extern crate glutin;
extern crate libc;
extern crate nanovg;
extern crate rustbox;


// use std::cell::Cell; // for glfw error count
// use std::ptr;
// use std::ffi::CString;
// use libc::*;
// use x11::xlib::*;


use rustbox::{Color, RustBox, Key};
use std::error::Error;
use std::default::Default;
mod backend;
use backend::*;

struct EditorView {
    pub editor: Editor,
    pub term: RustBox,
    x: usize,
    y: usize,
    row: usize,
    col: usize,
}

impl EditorView {
    fn new() -> EditorView {
        let term = match RustBox::init(Default::default()) {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}", e),
        };
        let col = term.width();
        let row = term.height();
        EditorView {
            editor: Editor::new(),
            term: term,
            x: 0,
            y: 0,
            row: row,
            col: col,
        }
    }
    fn redraw_line(&self, index: usize) {
        if self.editor.len() <= index {
            panic!("Line out of bounds!");
        }
        if index >= self.y && self.y + self.row >= index {
            let dy = index - self.y;
            for col in 4..self.col {
                self.term.print_char(col, dy, rustbox::RB_NORMAL, Color::White, Color::Black, ' ');
            }
            let line = self.editor.get(index).unwrap();
            if self.editor.main_caret.row == index {
                self.term.print(0, dy, rustbox::RB_BOLD, Color::Black, Color::White, " 1 ");
                self.term.print(4,
                                dy,
                                rustbox::RB_NORMAL,
                                Color::White,
                                Color::Black,
                                &line.extract());
            }
        }
    }
    fn draw_caret(&self) {
        self.term.set_cursor(self.editor.main_caret.col as isize + 4,
                             self.editor.main_caret.row as isize);
    }
}

fn main() {
    let mut view = EditorView::new();
    view.editor.insert(String::from("Hello world!!"));
    view.redraw_line(0);
    view.draw_caret();
    view.term.present();
    loop {
        match view.term.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Char(c) => {
                        view.editor.insert_char(c);
                        view.redraw_line(0);
                        view.draw_caret();
                        view.term.present();
                    }
                    Key::Enter => {
                        view.editor.insert_line();
                        view.redraw_line(0);
                        view.draw_caret();
                        view.term.present();
                    }
                    Key::Backspace => {
                        view.editor.backspace();
                        view.redraw_line(0);
                        view.draw_caret();
                        view.term.present();
                    }
                    Key::Left => {
                        view.editor.backward();
                        view.draw_caret();
                        view.term.present();
                    }
                    Key::Right => {
                        view.editor.forward();
                        view.draw_caret();
                        view.term.present();
                    }
                    Key::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
            Err(e) => panic!("{}", e.description()),
            _ => {}
        }
    }
}

// struct XIMDevice {
// im: XIM,
// ic: XIC,
// }
//
// impl XIMDevice {
// pub fn new(dsp: *mut Display, win: *mut Window) -> XIMDevice {
// unsafe {
// if setlocale(LC_CTYPE, CString::new("").unwrap().as_ptr()).is_null() {
// panic!("Can't set locale.");
// }
// if XSupportsLocale() == 0 {
// panic!("Current locale is not supported.");
// }
// if XSetLocaleModifiers(CString::new("").unwrap().as_ptr()).is_null() {
// panic!("Can't set locale modifiers.\n");
// }
// println!("open");
// let im: XIM = XOpenIM(dsp, ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
// println!("start check");
// if im.is_null() {
// panic!("Couldn't open input method.");
// }
// println!("not null");
// println!("{}", im.is_null());
// let ic: XIC = XCreateIC(im,
// XNInputStyle,
// XIMPreeditCallbacks | XIMStatusCallbacks,
// XNClientWindow,
// win);
// println!("open ic");
// if ic.is_null() {
// println!("null");
// XCloseIM(im);
// panic!("Couldn't create input context.");
// }
// println!("not null");
// XIMDevice { im: im, ic: ic }
// }
// }
// pub fn close(&mut self) {
// unsafe {
// XDestroyIC(self.ic);
// XCloseIM(self.im);
// }
// }
// }
//
// #[macro_use]
// mod util;
// fn init_gl() {
// glcheck!(unsafe { gl::FrontFace(gl::CCW) });
// glcheck!(unsafe { gl::Enable(gl::DEPTH_TEST) });
// glcheck!(unsafe { gl::Enable(gl::SCISSOR_TEST) });
// glcheck!(unsafe { gl::DepthFunc(gl::LEQUAL) });
// glcheck!(unsafe { gl::FrontFace(gl::CCW) });
// glcheck!(unsafe { gl::Enable(gl::CULL_FACE) });
// glcheck!(unsafe { gl::CullFace(gl::BACK) });
// }
//
//
// fn main() {
// let builder = glutin::WindowBuilder::new();
// let window = builder.with_dimensions(640, 480)
// .with_title("Test Window")
//                        .with_gl(Latest)
//                        .with_gl_profile(Core)
// .build()
// .unwrap();
//
// unsafe {
// window.make_current();
// }
// gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
//
// let mut xim = XIMDevice::new(unsafe { window.platform_display() } as _,
// unsafe { window.platform_window() } as _);
//
//
// init_gl();
//
// let vg: nanovg::Context = nanovg::Context::create_gl3(nanovg::ANTIALIAS |
//                                                      nanovg::STENCIL_STROKES);
//
// for event in window.wait_events() {
// unsafe { gl::Clear(gl::COLOR_BUFFER_BIT) };
// window.swap_buffers();
//
// match event {
// glutin::Event::Closed => break,
// _ => (),
// }
// }
//
// xim.close();
// }
//