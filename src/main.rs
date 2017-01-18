#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

extern crate clap;
extern crate termion;

use termion::{clear, color, style, cursor};
use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead, MouseTerminal};
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{Write, stdout, stdin};
use clap::{Arg, App};
use std::path::Path;
use std::io::{Stdin, Stdout};
mod backend;
use backend::*;

struct EditorView {
    pub editor: Editor,
    pub stdout: MouseTerminal<RawTerminal<Stdout>>,
    x: usize,
    y: usize,
    row: usize,
    col: usize,
    lnum_pad: usize,
}

fn right_aligned_text(text: &str, width: usize) -> String {
    let len = text.chars().count();
    if width < len {
        panic!("\"{}\" is out of width size {}!", text, width);
    }
    let mut aligned = String::with_capacity(width);
    for idx in 0..width - len {
        aligned.push(' ');
    }
    aligned.push_str(text);
    aligned
}

impl EditorView {
    fn new() -> EditorView {
        let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
        let size = termion::terminal_size().unwrap();
        let col = size.0 as usize;
        let row = size.1 as usize;
        EditorView {
            editor: Editor::new(),
            stdout: stdout,
            x: 0,
            y: 0,
            row: row - 1,
            col: col,
            lnum_pad: 1,
        }
    }
    fn clear(&mut self) {
        write!(self.stdout, "{}", clear::All);
    }
    fn format_info(&self) -> String {
        let main_caret = self.editor.carets.get(0).expect("Caret not found!");
        right_aligned_text(&format!("{}:{}  ", main_caret.row, main_caret.col),
                           self.col)
    }
    fn redraw(&mut self) {
        let start = self.y;
        let end = self.editor.len() - start;
        let height = self.row;
        for idx in start..start + height {
            self.redraw_line(idx);
        }
        self.redraw_infobar();
    }
    fn redraw_infobar(&mut self) {
        let info = self.format_info();
        write!(self.stdout,
               "{}{}{}{}{}{}{}",
               cursor::Goto(1, (self.row + 1) as u16),
               color::Fg(color::White),
               color::Bg(color::Rgb(50, 50, 50)),
               style::Bold,
               &info,
               color::Bg(color::Black),
               style::Reset);
    }
    fn redraw_line(&mut self, index: usize) {
        let main_caret = self.editor.carets.get(0).expect("Caret not found!");
        let dy = (index - self.y) as u16;
        // write!(self.stdout,
        //     "{}{}",
        //   cursor::Goto(1, dy + 1),
        // clear::CurrentLine);
        if self.editor.len() > index && index >= self.y && self.y + self.row >= index {
            let line = self.editor.get(index).unwrap();
            write!(self.stdout,
                   "{}{}{}{}{} {}{}{}",
                   cursor::Goto(1, dy + 1),
                   style::Bold,
                   color::Fg(color::Yellow),
                   color::Bg(color::Black),
                   &right_aligned_text(&(index + 1).to_string(), self.lnum_pad),
                   style::Reset,
                   color::Fg(color::White),
                   color::Bg(color::Black));
            if main_caret.row == index {
                let col = main_caret.col;
                let count = line.len();
                if col == count {
                    write!(self.stdout,
                           "{}{}{}{}",
                           &line.extract(),
                           style::Invert,
                           ' ',
                           style::Reset);
                } else {
                    for idx in 0..col {
                        write!(self.stdout, "{}", line[idx]);
                    }
                    let c = match line.get(col) {
                        Some(ch) => *ch,
                        None => ' ',
                    };
                    write!(self.stdout, "{}{}{}", style::Invert, c, style::Reset);
                    for idx in col + 1..count {
                        write!(self.stdout, "{}", line[idx]);
                    }
                }

            } else {
                write!(self.stdout,
                       "{}{}{}{}{} {}{}{}{}",
                       cursor::Goto(1, dy + 1),
                       style::Bold,
                       color::Fg(color::Yellow),
                       color::Bg(color::Black),
                       &right_aligned_text(&(index + 1).to_string(), self.lnum_pad),
                       style::Reset,
                       color::Fg(color::White),
                       color::Bg(color::Black),
                       &line.extract());
            }
        }
    }
    fn draw_caret(&mut self) {
        let main_caret = self.editor.carets.get(0).expect("Caret not found!");
        write!(self.stdout,
               "{}{}",
               cursor::Goto((main_caret.col - self.x + 2 + self.lnum_pad) as u16,
                            (main_caret.row - self.y + 1) as u16),
               cursor::Show);
    }
    fn flush(&mut self) {
        self.stdout.flush().unwrap();
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

    view.lnum_pad = view.editor.len().to_string().chars().count();

    let stdin = stdin();

    view.clear();
    view.redraw();
    view.draw_caret();
    view.flush();
    for c in stdin.events() {
        write!(view.stdout, "{}", clear::All);
        let evt = c.unwrap();
        match evt {
            Event::Key(key) => {
                match key {
                    Key::Char('\n') => {
                        view.editor.insert_line();
                        {
                            let main_caret = view.editor.carets.get(0).expect("Caret not found!");
                            if main_caret.row >= view.y + view.row {
                                view.y += 1;
                            }
                        }
                        view.lnum_pad = view.editor.len().to_string().chars().count();
                        view.redraw();
                    }

                    Key::Backspace => {
                        view.editor.backspace();
                        if view.editor.carets.get(0).expect("Caret not found!").row < view.y {
                            view.y -= 1;
                            view.redraw();
                        }
                        view.redraw();
                    }
                    Key::Home => {
                        view.y = 0;
                        view.editor.move_top();
                        view.redraw();
                    }
                    Key::End => {
                        let row = view.editor.len() - 1;
                        view.y = row;
                        view.editor.move_end();
                        view.redraw();
                    }
                    Key::PageUp => {
                        view.editor.move_pageup(view.row - 1);
                        let row = view.editor.carets.get(0).expect("Caret not found!").row;
                        view.y = row;
                        view.redraw();
                    }
                    Key::PageDown => {
                        view.editor.move_pagedown(view.row - 1);
                        let row = view.editor.carets.get(0).expect("Caret not found!").row + 1;
                        let len = view.editor.len();
                        if row > view.row {
                            view.y = row - view.row;
                        } else {
                            view.y = 0;
                        }
                        view.redraw();
                    }
                    Key::Left => {
                        view.editor.move_left();
                        view.redraw();
                    }
                    Key::Right => {
                        view.editor.move_right();
                        view.redraw();
                    }
                    Key::Up => {
                        view.editor.move_up();
                        {
                            let main_caret = view.editor.carets.get(0).expect("Caret not found!");
                            if main_caret.row < view.y {
                                view.y -= 1;
                            }
                        }
                        view.redraw();
                    }
                    Key::Down => {
                        view.editor.move_down();
                        {
                            let main_caret = view.editor.carets.get(0).expect("Caret not found!");
                            if main_caret.row >= view.y + view.row {
                                view.y += 1;
                            }
                        }
                        view.redraw();
                    }
                    Key::Esc => {
                        break;
                    }
                    Key::Char(c) => {
                        view.editor.insert_char(c);
                        view.redraw();
                    }
                    _ => {}
                }
            }
            Event::Mouse(me) => {
                match me {
                    MouseEvent::Press(_, x, y) => {
                        let main_caret = view.editor.carets.get_mut(0).expect("Caret not found!");
                        main_caret.col = x as usize;
                        main_caret.row = y as usize + view.row;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        view.redraw_infobar();
        view.draw_caret();
        view.flush();
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