#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

extern crate rustbox;
extern crate clap;

use rustbox::{Color, RustBox, Key};
use clap::{Arg, App};
use std::path::Path;
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
    lnum_pad: usize,
}

fn right_aligned_text(text: &str, width: usize) -> String {
    let blanks_len = width - text.chars().count();
    if blanks_len < 0 {
        panic!("{} is out of width size {}!", text, width);
    }
    let mut aligned = String::with_capacity(width);
    for idx in 0..blanks_len {
        aligned.push(' ');
    }
    aligned.push_str(text);
    aligned
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
            row: row - 1,
            col: col,
            lnum_pad: 1,
        }
    }
    fn format_info(&self) -> String {
        right_aligned_text(&format!("{}:{}  ",
                                    self.editor.main_caret.row,
                                    self.editor.main_caret.col),
                           self.col)
    }
    fn redraw(&self) {
        let start = self.y;
        let end = self.editor.len() - start;
        let height = self.row;
        for idx in start..start + height {
            self.redraw_line(idx);
        }
        self.redraw_infobar();
    }
    fn redraw_infobar(&self) {
        self.term.print(0,
                        self.row,
                        rustbox::RB_BOLD,
                        Color::White,
                        Color::Cyan,
                        &self.format_info());
    }
    fn redraw_line(&self, index: usize) {
        let dy = index - self.y;
        for col in 0..self.col {
            self.term.print_char(col, dy, rustbox::RB_NORMAL, Color::White, Color::Black, ' ');
        }
        if self.editor.len() > index && index >= self.y && self.y + self.row >= index {
            let line = self.editor.get(index).unwrap();
            self.term.print(0,
                            dy,
                            rustbox::RB_BOLD,
                            Color::Yellow,
                            Color::Black,
                            &format!("{} ",
                                     right_aligned_text(&(index + 1).to_string(), self.lnum_pad)));
            self.term.print(self.lnum_pad + 2,
                            dy,
                            rustbox::RB_NORMAL,
                            Color::White,
                            Color::Black,
                            &line.extract());

        }
    }
    fn draw_caret(&self) {
        self.term.set_cursor((self.editor.main_caret.col + self.lnum_pad) as isize + 2,
                             (self.editor.main_caret.row - self.y) as isize);
    }
}

fn main() {
    let matches = App::new("Mal")
                      .version("0.1.0")
                      .author("Tamamu <tamamu.1r1s@gmail.com>")
                      .about("Minimal text editor")
                      .arg(Arg::with_name("FILE")
                               .short("o")
                               .long("open")
                               .value_name("FILE")
                               .help("Sets the file to edit"))
                      .get_matches();

    let mut view = EditorView::new();
    // view.editor.insert(String::from("Hello world!!"));

    if let Some(path) = matches.value_of("FILE") {
        view.editor.read_file(Path::new(path))
    }


    view.redraw();
    view.draw_caret();
    view.term.present();
    loop {
        match view.term.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Char(c) => {
                        view.editor.insert_char(c);
                        view.redraw_line(view.editor.main_caret.row);
                        view.draw_caret();
                        view.term.present();
                    }
                    Key::Enter => {
                        view.editor.insert_line();
                        if view.editor.main_caret.row >= view.y + view.row {
                            view.y += 1;
                        }
                        view.lnum_pad = view.editor.len().to_string().chars().count();
                        view.redraw();
                        view.draw_caret();
                        view.term.present();
                    }
                    Key::Backspace => {
                        view.editor.backspace();
                        if view.editor.main_caret.row < view.y {
                            view.y -= 1;
                            view.redraw();
                        }
                        view.redraw();
                        view.draw_caret();
                        view.term.present();
                    }
                    Key::Left => {
                        view.editor.move_left();
                        view.draw_caret();
                        view.term.present();
                    }
                    Key::Right => {
                        view.editor.move_right();
                        view.draw_caret();
                        view.term.present();
                    }
                    Key::Up => {
                        view.editor.move_up();
                        if view.editor.main_caret.row < view.y {
                            view.y -= 1;
                            view.redraw();
                        }
                        view.draw_caret();
                        view.term.present();
                    }
                    Key::Down => {
                        view.editor.move_down();
                        if view.editor.main_caret.row >= view.y + view.row {
                            view.y += 1;
                            view.redraw();
                        }
                        view.draw_caret();
                        view.term.present();
                    }
                    Key::Esc => {
                        break;
                    }
                    _ => {}
                }
                view.redraw_infobar();
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