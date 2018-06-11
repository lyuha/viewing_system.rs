#[cfg(all(feature = "winit", feature = "glium"))]
#[macro_use]
extern crate conrod;

fn main() {
    feature::main();
}

#[cfg(all(feature = "winit", feature = "glium"))]
mod feature {
    extern crate find_folder;
    use conrod;
    use conrod::backend::glium::glium::{self, Surface};
    use std;

    pub fn main() {
        const WIDTH: u32 = 1280;
        const HEIGHT: u32 = 720;

        // Build the window.
        let mut events_loop = glium::glutin::EventsLoop::new();
        let window = glium::glutin::WindowBuilder::new()
            .with_title("Hello Conrod!")
            .with_dimensions(WIDTH, HEIGHT);
        let context = glium::glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4);
        let display = glium::Display::new(window, context, &events_loop).unwrap();

        // construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

        // Add a `font` to the `Ui`'s `font::Map` from file.
        const FONT_PATH: &'static str = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/fonts/NotoSans/NotoSans-Regular.ttf"
        );
        ui.fonts.insert_from_file(FONT_PATH).unwrap();

        // A type used for converting `conrod::render::Primitives` into `Command`s that can be used for drawing to the glium `Surface`.
        let mut renderer = conrod::backend::glium::Renderer::new(&display).unwrap();

        // The image map describing each of our widget->image_mappings (in our case, none).
        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        // Instantiate the generated list of widget identifiers.
        let ids = &mut Ids::new(ui.widget_id_generator());

        let mut events = Vec::new();

        'render: loop {
            events.clear();

            // Get all the new events since the last frames.
            events_loop.poll_events(|event| {
                events.push(event);
            });

            // If there are no new events, wait for one.
            if events.is_empty() {
                events_loop.run_forever(|event| {
                    events.push(event);
                    glium::glutin::ControlFlow::Break
                });
            }

            // Process the events
            for event in events.drain(..) {
                // Break from the loop upon `Escape` or closed window.
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

                // Instantiate all widgets in the GUI.

                // Set the widgets.
                set_widgets(ui.set_widgets(), ids);
            }

            // Draw the `Ui` if it has changed.
            if let Some(primitives) = ui.draw_if_changed() {
                renderer.fill(&display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color(0.0, 0.0, 0.0, 1.0);
                renderer.draw(&display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    }

    // Draw the Ui.
    fn set_widgets(ref mut ui: conrod::UiCell, ids: &mut Ids) {
        use conrod::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget};

        // Construct our main `Canvas` tree
        widget::Canvas::new()
            .flow_left(&[
                (
                    ids.left_column,
                    widget::Canvas::new().color(color::BLACK).pad(10.0),
                ),
                (ids.right_column, widget::Canvas::new().color(color::WHITE)),
            ])
            .set(ids.master, ui);

        // "Viewing System!" in the middle of the left column.
        widget::Text::new("Viewing System")
            .middle_of(ids.left_column)
            .color(color::WHITE)
            .font_size(32)
            .set(ids.text_view, ui);

        widget::Text::new("Hello, conrod!")
            .middle_of(ids.right_column)
            .color(color::BLACK)
            .font_size(32)
            .set(ids.text_conrod, ui);
    }

    widget_ids!(
        struct Ids {
            master,
            left_column,
            right_column,
            text_view,
            text_conrod,
        }
    );
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
