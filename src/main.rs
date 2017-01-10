#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

extern crate x11;
extern crate gl;
extern crate glfw;
extern crate libc;
extern crate nanovg;


use glfw::Context as GlfwContext;
use std::cell::Cell; // for glfw error count
use std::ptr;
use std::ffi::CString;
use libc::*;
use x11::xlib::*;
use glfw::{Action, Context, Key};

struct XIMDevice {
    im: XIM,
    ic: XIC,
}

impl XIMDevice {
    pub fn new(dsp: *mut Display, win: *mut c_void) -> XIMDevice {
        unsafe {
            if setlocale(LC_CTYPE, CString::new("").unwrap().as_ptr()).is_null() {
                panic!("Can't set locale.");
            }
            if XSupportsLocale() == 0 {
                panic!("Current locale is not supported.");
            }
            if XSetLocaleModifiers(CString::new("").unwrap().as_ptr()).is_null() {
                panic!("Can't set locale modifiers.\n");
            }
            let im: XIM = XOpenIM(dsp,
                                  ptr::null_mut(),
                                  ptr::null_mut(),
                                  ptr::null_mut());
            if im.is_null() {
                panic!("Couldn't open input method.");
            }
            let ic: XIC = XCreateIC(im,
                                    XNInputStyle,
                                    XIMPreeditNothing | XIMStatusNothing,
                                    XNClientWindow,
                                    win);
            if ic.is_null() {
                XCloseIM(im);
                panic!("Couldn't create input context.");
            }
            XIMDevice {
                im: im,
                ic: ic,
            }
        }
    }
    pub fn close(&mut self) {
        unsafe {
            XDestroyIC(self.ic);
            XCloseIM(self.im);
        }
    }
}

#[macro_use]
mod util;
use util::*;
fn init_gl() {
    glcheck!(unsafe {gl::FrontFace(gl::CCW)});
    glcheck!(unsafe {gl::Enable(gl::DEPTH_TEST)});
    glcheck!(unsafe {gl::Enable(gl::SCISSOR_TEST)});
    glcheck!(unsafe {gl::DepthFunc(gl::LEQUAL)});
    glcheck!(unsafe {gl::FrontFace(gl::CCW)});
    glcheck!(unsafe {gl::Enable(gl::CULL_FACE)});
    glcheck!(unsafe {gl::CullFace(gl::BACK)});
}


fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 2));
 	glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
 	glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));

    let (mut window, events) = glfw.create_window(640, 480, "Title",
                                                  glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

//    let mut xim = XIMDevice::new(glfw.get_x11_display() as _,
//                                 window.get_x11_window() as _);

    window.set_key_polling(true);
    window.make_current();

    glcheck!(gl::load_with(|name| window.get_proc_address(name) as *const _));
    init_gl();
    
    let vg: nanovg::Context = nanovg::Context::create_gles2(nanovg::ANTIALIAS | nanovg::STENCIL_STROKES);

    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut window, event);
        }
    }

//    xim.close();
}

fn handle_window_event(window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
            window.set_should_close(true)
        }
        _ => {}
    }
}
