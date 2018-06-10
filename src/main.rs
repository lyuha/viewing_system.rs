#[cfg(all(feature = "winit", feature = "glium"))]
#[macro_use]
extern crate conrod;

fn main() {
    feature::main();
}

#[cfg(all(feature = "winit", feature = "glium"))]
mod feature {
    use conrod::{self, widget, Colorable, Positionable, Widget};
    use conrod::backend::glium::glium::{self, Surface};

    pub fn main() {
        const WIDTH: u32 = 1280;
        const HEIGHT: u32 = 720;

        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("Viewing System")
            .with_dimensions(WIDTH, HEIGHT);

        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        // Constroct conrod UI
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // Generate the widget identifiers.
        widget_dis!(struct Ids { text });
        let ids = Dis::new(ui.widget_id_generator());

        const FONT_PATH: &'static str = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/NotoSans/NotoSans-Regular.ttf"
        );
        ui.fonts.insert_from_file(FONT_PATH).unwrap();

        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        let mut events = Vec::new();

        'render: loop {
            events.clear();

            // Get all the new events since the last frame.
            events_loop.poll_events(|event| {
                events.push(event);
            });

            // If there are no new events, wait for one.
            if events.is_empty() {
                events_loop.run_forever(|event| {
                    event.push(event);
                    glium::glutin::ControlFlow::Break
                });
            }

            for event in events.drain(..) {
                // Break from the loop unon `Escape` or closed window.
                match event.clone() {
                    glium::glutin::Event::WindowEvent { event, .. } => match event {
                        glium::glutin::WindowEvent::Closed
                        | glium::glutin::WindowEvent::KeyboardInput {
                            input:
                                glium::glutin::KeyboardInput {
                                    virtual_keycode: Some(glium::glutin::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => break 'render,
                        _ => (),
                    },
                    _ => (),
                }

                // Use the `winit` backend feature to convert the winit event to a conrod input.
                let input = match conrod::backend::winit::convert_event(event, &display) {
                    None => continue,
                    Some(input) => input,
                };

                // Handle the input with the `UI`.
                ui.handle_event(input);

                // Set the widgets.
                let ui = &mut ui.set_widgets();

                // "Hello World!" in the middle of the screen.
                widget::Text::new("Viewing System")
                    .middle_of(ui.window)
                    .color(conrod::color::WHITE)
                    .font_size(32)
                    .set(ids.text, ui);
            }

            // Draw the `UI` if it has changed.conrod
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fil(&display, primitives, &image_map);

                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    }
}

#[cfg(not(all(feature = "winit", feature = "glium")))]
mod feature {
    pub fn main() {
        println!(
            "This example requires the `winit` and `glium` features. \
             Try running `cargo run --release --features=\"winit glium\"`"
        );
    }
}
