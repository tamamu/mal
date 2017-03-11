#![feature(core)]
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

extern crate clap;
extern crate rustbox;

use std::default::Default;
use std::io::{Write, stdout, stdin};
use clap::{Arg, App};
use rustbox::{Color, RustBox, Key, OutputMode};
use std::path::Path;
use std::io::{Stdin, Stdout};
mod backend;
use backend::*;

struct EditorView {
    pub editor: Editor,
    pub terminal: RustBox,
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
        let mut terminal = match RustBox::init(Default::default()) {
            Result::Ok(v) => v,
            Result::Err(e) => panic!("{}", e),
        };
        // terminal.set_output_mode(OutputMode::EightBit);
        let col = terminal.width();
        let row = terminal.height();
        EditorView {
            editor: Editor::new(),
            terminal: terminal,
            x: 0,
            y: 0,
            row: row - 1,
            col: col,
            lnum_pad: 1,
        }
    }
    fn clear(&mut self) {
        self.terminal.clear();
    }
    fn format_info(&self) -> String {
        let main_caret = self.editor.carets.get(0).expect("Caret not found!");
        right_aligned_text(&format!("{}:{}", main_caret.row + 1, main_caret.col + 1),
                           self.col)
    }
    fn redraw(&mut self) {
        self.clear();
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
        self.terminal.print(0,
                            self.row,
                            rustbox::RB_NORMAL,
                            Color::White,
                            Color::Blue,
                            &info);
    }
    fn redraw_line(&mut self, index: usize) {
        let main_caret = self.editor.carets.get(0).expect("Caret not found!");
        let dy = index - self.y;
        // write!(self.stdout,
        //     "{}{}",
        //   cursor::Goto(1, dy + 1),
        // clear::CurrentLine);
        if self.editor.len() > index && index >= self.y && self.y + self.row >= index {
            let line = self.editor.get(index).unwrap();
            self.terminal.print(0,
                                dy,
                                rustbox::RB_BOLD,
                                Color::Yellow,
                                Color::Black,
                                &right_aligned_text(&(index + 1).to_string(), self.lnum_pad));
            if main_caret.row == index {
                let col = main_caret.col;
                let count = line.len();
                if col == count {
                    self.terminal.print(self.lnum_pad + 1,
                                        dy,
                                        rustbox::RB_NORMAL,
                                        Color::White,
                                        Color::Black,
                                        &line.extract());
                    self.terminal.print_char(self.lnum_pad + count + 1,
                                             dy,
                                             rustbox::RB_REVERSE,
                                             Color::White,
                                             Color::Black,
                                             ' ');
                } else {
                    for idx in 0..col {
                        self.terminal.print_char(self.lnum_pad + idx + 1,
                                                 dy,
                                                 rustbox::RB_NORMAL,
                                                 Color::White,
                                                 Color::Black,
                                                 line[idx]);
                    }
                    let c = match line.get(col) {
                        Some(ch) => *ch,
                        None => ' ',
                    };
                    self.terminal.print_char(self.lnum_pad + col + 1,
                                             dy,
                                             rustbox::RB_REVERSE,
                                             Color::White,
                                             Color::Black,
                                             c);
                    let ranged_col = col + main_caret.range as usize;
                    for idx in col + 1..ranged_col + 1 {
                        self.terminal.print_char(self.lnum_pad + idx + 1,
                                                 dy,
                                                 rustbox::RB_REVERSE,
                                                 Color::White,
                                                 Color::Black,
                                                 line[idx]);
                    }
                    for idx in ranged_col + 1..count {
                        self.terminal.print_char(self.lnum_pad + idx + 1,
                                                 dy,
                                                 rustbox::RB_NORMAL,
                                                 Color::White,
                                                 Color::Black,
                                                 line[idx]);
                    }
                }

            } else {
                self.terminal.print(self.lnum_pad + 1,
                                    dy,
                                    rustbox::RB_NORMAL,
                                    Color::White,
                                    Color::Black,
                                    &line.extract());
            }
        }
    }
    fn draw_caret(&mut self) {
        let main_caret = self.editor.carets.get(0).expect("Caret not found!");
        self.terminal.set_cursor((main_caret.col - self.x + 1 + self.lnum_pad) as isize,
                                 (main_caret.row - self.y) as isize);
    }
    fn flush(&mut self) {
        self.terminal.present();
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

    // view.editor.mode_select();

    view.lnum_pad = view.editor.len().to_string().chars().count();

    view.clear();
    view.redraw();
    // view.draw_caret();
    view.flush();
    loop {
        match view.terminal.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(key)) => {
                match key {
                    Key::Enter => {
                        view.editor.insert_line();
                        {
                            let main_caret = view.editor
                                                 .carets
                                                 .get(0)
                                                 .expect("Caret not found!");
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
                            let main_caret = view.editor
                                                 .carets
                                                 .get(0)
                                                 .expect("Caret not found!");
                            if main_caret.row < view.y {
                                view.y -= 1;
                            }
                        }
                        view.redraw();
                    }
                    Key::Down => {
                        view.editor.move_down();
                        {
                            let main_caret = view.editor
                                                 .carets
                                                 .get(0)
                                                 .expect("Caret not found!");
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
            Ok(rustbox::Event::MouseEvent(mouse, x, y)) => {
                let main_caret = view.editor
                                     .carets
                                     .get_mut(0)
                                     .expect("Caret not found!");
                main_caret.col = x as usize;
                main_caret.row = y as usize + view.row;
            }
            _ => {}
        }
        view.redraw_infobar();
        // view.draw_caret();
        view.flush();
    }
}
