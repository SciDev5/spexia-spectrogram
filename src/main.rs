use audio::{DeviceSelector, Streamer};
use render::Window;
use util::{GenericResult, Vec2I};

mod audio;
mod render;
mod util;

fn main() -> GenericResult<()> {
    println!("{:?}", cpal::available_hosts());
    let mut audio_device_selector = DeviceSelector::new(false);
    let mut audio = Streamer::begin(&audio_device_selector)?;

    //// initialize rendering ////
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    let mut window = Window::new(
        &mut glfw,
        Vec2I(1024, 1024),
        false,
        true,
        "spexia spectrogram :3",
        |window| {
            window.set_key_polling(true);
        },
    );
    let mut render_app = render::RenderApp::new();

    let mut input_changed = false;
    //// program loop ////
    while !window.should_close() {
        //// audio system updates ////
        if input_changed || audio_device_selector.poll_device_has_changed(audio.did_lose_device()) {
            audio.update_stream(&audio_device_selector);
            println!("updated stream");
            input_changed = false;
        }
        // let mut updated = false;
        {
            let mut audio_data = audio.data.lock().unwrap();
            while let Some(k) = audio_data.take() {
                render_app.set_wave(&k, audio_data.sample_rate);
            }
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
                        glfw::Key::M => {
                            audio_device_selector.set_uses_input(!audio_device_selector.uses_input());
                            input_changed = true;
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
        // i += 1;
        // i %= 100;
        // if i != 0 {
        //     continue;
        // }
        window.render(|winfo| render_app.draw(&winfo));
    }

    Ok(())
}
