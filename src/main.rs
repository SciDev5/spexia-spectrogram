use audio::{Streamer, DeviceSelector};
use glfw::Context;
use util::GenericResult;

mod audio;
mod render;
mod util;

fn main() -> GenericResult<()> {
    let mut audio_device_selector = DeviceSelector::new(false);
    let mut audio = Streamer::begin(&audio_device_selector)?;

    // Initialize GLFW
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

    glfw.window_hint(glfw::WindowHint::Floating(false));
    glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));

    let (mut window, events) = glfw
        .create_window(1024, 1024, "spexia viewer :3", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window");

    window.make_current();
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    // Initialize OpenGL
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    unsafe {
        gl::Viewport(0, 0, 1024, 1024);
        gl::ClearColor(0.0, 0.0, 0.0, 0.0);
        gl::Disable(gl::DEPTH_TEST);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::BlendEquation(gl::FUNC_ADD);
    }

    let mut h = render::H::new(window.get_size().1);

    window.set_key_polling(true);
    window.set_size_polling(true);
    
    let mut fullscreen = false;

    // Render loop
    while !window.should_close() {
        // Poll stuff.
        glfw.poll_events();
        if audio_device_selector.poll_device_has_changed(audio.did_lose_device()) {
            audio.update_stream(&audio_device_selector);
        }

        // Pass audio data to renderer/visualizer
        while let Some(k) = audio.data.lock().unwrap().take() {
            h.set_wave(&k);
        }

        // Obvious
        unsafe {
            h.draw();
        }

        // Handle events
        while let Ok((_,ev)) = events.try_recv() {
            match ev {
                glfw::WindowEvent::Key(key, _scancode, action, _modifiers) => {
                    if let glfw::Action::Press = action {
                        match key {
                            glfw::Key::D => {
                                window.set_decorated(!window.is_decorated());
                            }
                            glfw::Key::F => {
                                fullscreen = !fullscreen;
                                if fullscreen {
                                    window.set_decorated(false);
                                    window.maximize();
                                } else {
                                    window.set_decorated(true);
                                    window.restore();
                                }
                            }
                            glfw::Key::T => {
                                window.set_floating(!window.is_floating());
                                h.window_is_floating = window.is_floating();
                            }
                            _ => {}
                        }
                    }
                }
                glfw::WindowEvent::Size(width, height) => {
                    unsafe { gl::Viewport(0, 0, width, height); }
                    h.window_height = height;
                    h.aspect = height as f32 / width as f32;
                }
                _ => {},
            }
        }

        // Finish frame
        window.swap_buffers();
    }

    Ok(())
}
