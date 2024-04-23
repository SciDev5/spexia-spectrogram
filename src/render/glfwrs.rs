use crate::util::{RectI, Vec2I};

use super::glrs;
use glfw::Context;

pub struct Window {
    window: glfw::PWindow,
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    winfo: Winfo,
}

impl Window {
    pub fn new<F: Fn(&mut glfw::PWindow)>(
        glfw: &mut glfw::Glfw,
        dim: Vec2I,
        floating: bool,
        decorated: bool,
        name: &str,
        configure: F,
    ) -> Window {
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        glfw.window_hint(glfw::WindowHint::Floating(floating));
        glfw.window_hint(glfw::WindowHint::Decorated(decorated));
        glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));

        let (mut window, events) = glfw
            .create_window(dim.0 as u32, dim.1 as u32, name, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");

        window.make_current();
        glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        configure(&mut window);

        Window {
            winfo: Winfo {
                bounds: RectI {
                    pos: window.get_pos().into(),
                    dim,
                },
                floating,
                decorated,
            },
            window,
            events,
        }
    }

    pub fn render<F: Fn(Winfo)>(&mut self, render: F) {
        //////////////////////////////////////////
        self.winfo = Winfo {
            bounds: RectI {
                pos: self.window.get_pos().into(),
                dim: self.window.get_size().into(),
            },
            floating: self.window.is_floating(),
            decorated: self.window.is_decorated(),
        };
        //////////////////////////////////////////
        if !self.window.is_current() {
            self.window.make_current()
        }
        glrs::viewport(RectI {
            pos: Vec2I(0, 0),
            dim: self.winfo.bounds.dim,
        });
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
        }
        //////////////////////////////////////////
        render(self.winfo);
        //////////////////////////////////////////
        self.window.swap_buffers();
        //////////////////////////////////////////
    }

    pub fn handle_events<F: Fn(glfw::WindowEvent, &mut Winfo)>(&mut self, f: F) {
        let mut winfo = self.winfo;
        while let Some((_, ev)) = self.events.receive() {
            f(ev, &mut winfo);
        }
        self.update_from_winfo(winfo);
    }

    fn update_from_winfo(&mut self, winfo: Winfo) {
        if winfo.bounds.pos != self.winfo.bounds.pos {
            let Vec2I(x, y) = winfo.bounds.pos;
            self.window.set_pos(x, y);
        }
        if winfo.bounds.dim != self.winfo.bounds.dim {
            let Vec2I(w, h) = winfo.bounds.dim;
            self.window.set_size(w, h);
        }
        if winfo.decorated != self.winfo.decorated {
            self.window.set_decorated(winfo.decorated);
        }
        if winfo.floating != self.winfo.floating {
            self.window.set_floating(winfo.floating);
        }
        self.winfo = winfo;
    }

    pub fn should_close(&self) -> bool {
        self.window.should_close()
    }
}

/// Window information ("winfo" for short)
#[derive(Debug, Clone, Copy)]
pub struct Winfo {
    pub bounds: RectI,
    pub floating: bool,
    pub decorated: bool,
}
