use audio::Streamer;
use glfw::Context;
use util::GenericResult;

mod audio;
mod render;
mod util;

fn main() -> GenericResult<()> {
    let audio = Streamer::begin()?;

    // Initialize GLFW
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    // glfw.window_hint(glfw::WindowHint::TransparentFramebuffer(true));

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

    let mut h = render::H::new();

    window.set_key_polling(true);

    // Render loop
    while !window.should_close() {
        glfw.poll_events();

        while let Some(k) = audio.data.lock().unwrap().take() {
            h.set_wave(&k);
        }

        unsafe {
            h.draw();
        }

        while let Ok((_,ev)) = events.try_recv() {
            match ev {
                glfw::WindowEvent::Key(key, _scancode, action, _modifiers) => {
                    if let glfw::Key::D = key {
                        if let glfw::Action::Press = action {
                            window.set_decorated(!window.is_decorated());
                        }
                    }
                }
                _ => {},
            }
        }
        window.swap_buffers();
    }

    Ok(())
}
