use audio::{DeviceSelector, Streamer};
use render::Window;
use util::{GenericResult, Vec2I};

mod audio;
mod render;
mod util;

fn main() -> GenericResult<()> {
    let mut audio_device_selector = DeviceSelector::new(false);
    let mut audio = Streamer::begin(&audio_device_selector)?;

    //// initialize rendering ////
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    let mut window = Window::new(
        &mut glfw,
        Vec2I(512, 1024),
        false,
        true,
        "spexia spectrogram :3",
        |window| {
            window.set_key_polling(true);
        },
    );
    let mut render_app = render::RenderApp::new();

    //// program loop ////
    while !window.should_close() {
        //// audio system updates ////
        if audio_device_selector.poll_device_has_changed(audio.did_lose_device()) {
            audio.update_stream(&audio_device_selector);
        }
        while let Some(k) = audio.data.lock().unwrap().take() {
            render_app.set_wave(&k);
        }

        //// window polling and events ////
        glfw.poll_events();
        window.handle_events(|ev, winfo| match ev {
            glfw::WindowEvent::Key(key, _scancode, action, _modifiers) => {
                if let glfw::Action::Press = action {
                    match key {
                        glfw::Key::D => {
                            winfo.decorated = !winfo.decorated;
                        }
                        glfw::Key::T => {
                            winfo.floating = !winfo.floating;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        });

        //// render ////
        window.render(|winfo| render_app.draw(&winfo));
    }

    Ok(())
}
