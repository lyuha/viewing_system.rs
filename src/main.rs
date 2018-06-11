#[cfg(all(feature = "winit", feature = "glium"))]
#[macro_use]
extern crate conrod;

fn main() {
    feature::main();
}

#[cfg(all(feature = "winit", feature = "glium"))]
mod feature {
    extern crate find_folder;
    extern crate nfd;
    extern crate image;

    use conrod::{self, widget, Sizeable, Positionable, Widget};
    use conrod::backend::glium::glium::{self, Surface};
    use std;
    use self::image::Pixel;

    const SCREEN_WIDTH: u32 = 900;

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
        let mut image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        // Instantiate the generated list of widget identifiers.
        let ids = &mut Ids::new(ui.widget_id_generator());


        // let screen = image_to_texture2d(image::RgbaImage::new(SCREEN_WIDTH, HEIGHT), &display);
        let screen = image_to_texture2d(image::RgbaImage::from_fn(SCREEN_WIDTH, HEIGHT, |x, y| {
            if x % 2 == 0 {
                image::Luma([0u8]).to_rgba()
            } else {
                image::Luma([255u8]).to_rgba()
            }
        }), &display);
        let (screen_w, screen_h) = screen.dimensions();

        let screen = image_map.insert(screen);


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
                {
                    let ui = &mut ui.set_widgets();
                    set_widgets(ui, ids);
                    widget::Image::new(screen).w_h(screen_w as conrod::Scalar, screen_h as conrod::Scalar).middle_of(ids.draw_panel).set(ids.draw_texture, ui);
                }
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
    fn set_widgets(ui: &mut conrod::UiCell, ids: &mut Ids) {
        use self::nfd::Response;

        use conrod::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget};

        // Construct our main `Canvas` tree
        widget::Canvas::new()
            .flow_right(&[
                (
                    ids.draw_panel,
                    widget::Canvas::new().color(color::BLACK).length(SCREEN_WIDTH as f64),
                ),
                (
                    ids.control_panel,
                    widget::Canvas::new()
                        .color(color::WHITE)
                        .flow_down(&[
                            (
                                ids.button_panel,
                                widget::Canvas::new()
                                    .color(color::DARK_GRAY)
                                    .length(100.0)
                                    .pad(20.0),
                            ),
                            (ids.position_panel, widget::Canvas::new().color(color::GRAY)),
                        ]),
                ),
            ])
            .set(ids.master, ui);

        widget::Tabs::new(&[
            (ids.tab_object, "Object"),
            (ids.tab_camera, "Camera"),
            (ids.tab_light, "Light"),
        ]).wh_of(ids.position_panel)
            .color(color::LIGHT_BLUE)
            .label_color(color::DARK_ORANGE)
            .label_font_size(32)
            .middle_of(ids.position_panel)
            .set(ids.tabs, ui);

        const BUTTON_DIMENSION: conrod::Dimensions = [100.0, 50.0];
        const MARGIN: conrod::Scalar = 30.0;

        for _press in widget::Button::new()
            .label("Open")
            .wh(BUTTON_DIMENSION)
            .mid_left_of(ids.button_panel)
            .set(ids.button_open, ui)
        {
            let current_path = std::env::current_dir();
            let path = current_path.as_ref().unwrap().to_str().unwrap();

            println!("{:?}", path);

            let result = nfd::dialog()
                .filter("dat")
                .default_path(path)
                .open()
                .unwrap_or_else(|e| panic!(e));

            match result {
                Response::Okay(file_path) => println!("File Path = {:?}", file_path),
                Response::OkayMultiple(files) => println!("Files {:?}", files),
                Response::Cancel => println!("User canceled"),
            }
        }

        for _press in widget::Button::new()
            .label("Reset")
            .wh(BUTTON_DIMENSION)
            .align_middle_y_of(ids.button_open)
            .align_middle_x_of(ids.button_panel)
            .parent(ids.button_panel)
            .set(ids.button_reset, ui)
        {
            println!("Press Reset button.");
        }

        let mut view_mode = ViewMode::Parallel;
        let is_perspective = view_mode == ViewMode::Perspective;
        let view_label = if is_perspective {
            "Perspective"
        } else {
            "Parallel"
        };
        for _press in widget::Toggle::new(is_perspective)
            .label(view_label)
            .wh(BUTTON_DIMENSION)
            .mid_right_of(ids.button_panel)
            .align_middle_y_of(ids.button_open)
            .color(color::WHITE)
            .set(ids.toggle_view_mode, ui)
        {
            view_mode = if is_perspective {
                ViewMode::Parallel
            } else {
                ViewMode::Perspective
            };

        }
    }

    #[derive(PartialEq, Debug)]
    enum ViewMode {
        Parallel,
        Perspective,
    }

    widget_ids!(
        struct Ids {
            master,
            draw_panel,
            control_panel,
            button_panel,
            position_panel,

            draw_texture,

            button_open,
            button_reset,
            toggle_view_mode,

            tabs,
            tab_object,
            tab_camera,
            tab_light,

            label_object,
            label_camera,
            label_light,
        }
    );

    fn image_to_texture2d(image: image::RgbaImage, display: &glium::Display) -> glium::texture::Texture2d {
        let image_dimensions = image.dimensions();
        let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();
        texture
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
