use piston_window::*;

use pingui::GuiElement;
use pingui::{HAlign, Offset, VAlign};

struct World {
    pub arms: Vec<(f64, f64)>,
}

impl World {
    fn new() -> World {
        World { arms: Vec::new() }
    }

    fn add_arm(&mut self, length: f64, velocity: f64) {
        self.arms.push((length, velocity));
        println!("{:?}", self.max_length());
    }

    fn get_positions(&self, time: f64) -> Vec<(f64, f64)> {
        let mut origin = (0.0, 0.0);
        let mut positions = vec![origin];

        for i in 0..self.arms.len() {
            let (length, velocity) = self.arms[i];

            let x = (time * velocity).to_radians().cos() * length;
            let y = (time * velocity).to_radians().sin() * length;

            origin.0 += x;
            origin.1 += y;
            positions.push(origin);
        }

        positions
    }

    // TODO: Actually compute the max height/width
    fn max_length(&self) -> (f64, f64) {
        (
            self.arms.iter().fold(0.0, |sum, (length, _)| sum + length),
            self.arms.iter().fold(0.0, |sum, (length, _)| sum + length),
        )
    }
}

fn main() {
    let mut window: PistonWindow = WindowSettings::new("Hello Piston!", [640, 480])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut gui = pingui::Gui::new(window.factory.clone());
    let mut add_button = pingui::Button::new(
        &"+",
        Offset {
            align: (HAlign::Left, VAlign::Top),
            relative: (10.0, 25.0),
        },
    );
    let mut remove_button = pingui::Button::new(
        &"-",
        Offset {
            align: (HAlign::Left, VAlign::Top),
            relative: (30.0, 25.0),
        },
    );
    let mut clear_button = pingui::Button::new(
        &"Clear",
        Offset {
            align: (HAlign::Left, VAlign::Top),
            relative: (50.0, 25.0),
        },
    );
    let mut simulation_label = pingui::Label::new(
        &"Simulation",
        Offset {
            align: (HAlign::Right, VAlign::Top),
            relative: (10.0, 25.0),
        },
    );

    let input_start = Offset {
        align: (HAlign::Left, VAlign::Top),
        relative: (10.0, 55.0),
    };
    let mut arms_input: Vec<pingui::MultiBox> = Vec::new();

    let mut world = World::new();
    let mut draw_image = ::image::RgbaImage::new(1, 1);
    let mut draw_texture = Texture::from_image(
        &mut window.factory,
        &draw_image,
        &TextureSettings::new().filter(Filter::Nearest),
    )
    .unwrap();

    let mut new_trace = vec![(0.0, 0.0)];
    let mut positions = Vec::new();
    let mut time = 0.0;
    let mut speed = 1;
    let mut running = true;
    let mut update_image = false;
    while let Some(e) = window.next() {
        gui.event(&e);

        if update_image {
            let dimensions = world.max_length();
            draw_image = ::image::RgbaImage::new(
                dimensions.0.ceil() as u32 * 2 + 10,
                dimensions.1.ceil() as u32 * 2 + 10,
            );
            draw_texture = Texture::from_image(
                &mut window.factory,
                &draw_image,
                &TextureSettings::new()
                    .filter(Filter::Linear)
                    .mag(Filter::Linear),
            )
            .unwrap();
            update_image = false;
        }

        let offset_x = window.size().width / 2.0;
        let offset_y = window.size().height / 2.0;
        if let Some(args) = e.update_args() {
            if running {
                let iterations = speed as usize;
                for _ in 0..iterations {
                    positions = world.get_positions(time);
                    new_trace.push(*positions.last().unwrap());
                    time += args.dt * speed as f64 / iterations as f64;
                }
            } else {
                // This is needed to update the graphics while the simulation is stopped
                // And a new arm is added
                positions = world.get_positions(time);
            }
        }

        if let Some(button) = e.press_args() {
            if let Button::Keyboard(key) = button {
                match key {
                    Key::W => {
                        if speed < 2usize.pow(12) {
                            speed *= 2;
                        }
                    }
                    Key::S => {
                        if speed > 1 {
                            speed /= 2;
                        } else {
                            speed = 1;
                        }
                    }
                    Key::Space => {
                        running = !running;
                    }
                    Key::Q => {
                        save(&draw_image);
                    }
                    _ => {}
                }
            }
        }

        if let Some(_args) = e.render_args() {
            // Update the trace
            for i in 0..new_trace.len() - 1 {
                let a = new_trace[i];
                let b = new_trace[i + 1];
                let a = (
                    a.0 + draw_image.width() as f64 / 2.0 + 0.5,
                    a.1 + draw_image.height() as f64 / 2.0 + 0.5,
                );
                let b = (
                    b.0 + draw_image.width() as f64 / 2.0 + 0.5,
                    b.1 + draw_image.height() as f64 / 2.0 + 0.5,
                );
                draw_line(&mut draw_image, &a, &b, [64, 64, 64, 255]);
            }

            // Reappend the last point
            // So the next trace starts from where the last one left off
            let last = *new_trace.last().unwrap();
            new_trace = vec![last];

            draw_texture
                .update(&mut window.encoder, &draw_image)
                .unwrap();

            window.draw_2d(&e, |c, g| {
                clear([1.0; 4], g);

                // Draw the trace
                image(
                    &draw_texture,
                    c.transform.trans(
                        offset_x - draw_image.width() as f64 / 2.0,
                        offset_y - draw_image.height() as f64 / 2.0,
                    ),
                    g,
                );

                if !positions.is_empty() {
                    // Draw the arms
                    for i in 0..positions.len() - 1 {
                        let a = positions[i];
                        let b = positions[i + 1];

                        let line_data = [
                            a.0 + offset_x,
                            a.1 + offset_y,
                            b.0 + offset_x,
                            b.1 + offset_y,
                        ];
                        line([1.0, 0.1, 0.1, 1.0], 1.0, line_data, c.transform, g);
                    }
                }

                // GUI
                if clear_button.is_pressed() {
                    time = 0.0;
                    for pixel in draw_image.pixels_mut() {
                        *pixel = ::image::Rgba([0, 0, 0, 0]);
                    }
                    new_trace.clear();
                }

                for (i, arm_input) in &mut arms_input.iter().enumerate() {
                    let (mut length, mut velocity) = world.arms[i];

                    arm_input.input(0, &mut length);
                    arm_input.input(1, &mut velocity);

                    if (length, velocity) != world.arms[i] {
                        world.arms[i] = (length, velocity);
                        update_image = true;
                    }
                }

                if add_button.is_pressed() {
                    let mut offset = input_start.clone();
                    offset.relative.1 += 30.0 * arms_input.len() as f64;

                    let name = format!("Arm {}", arms_input.len());
                    let inputbox = pingui::MultiBox::new(&name, 10.0, 2, offset)
                        .value(0, &25.0)
                        .value(1, &90.0);

                    arms_input.push(inputbox);
                    world.add_arm(25.0, 90.0);
                    update_image = true;
                }
                if remove_button.is_pressed() {
                    arms_input.pop();
                    world.arms.pop();
                    update_image = true;
                }

                simulation_label.title = format!("Sim speed: {}x", speed);

                add_button.render(&mut gui, &c, g);
                remove_button.render(&mut gui, &c, g);
                clear_button.render(&mut gui, &c, g);
                simulation_label.render(&mut gui, &c, g);
                for input in &mut arms_input {
                    input.render(&mut gui, &c, g);
                }
            });
        }
    }
}

fn draw_line(image: &mut ::image::RgbaImage, from: &(f64, f64), to: &(f64, f64), color: [u8; 4]) {
    use imageproc::drawing;

    let line_thickness = 1.5;

    let delta = (to.0 - from.0, to.1 - from.1);
    let length = (delta.0.powi(2) + delta.1.powi(2)).sqrt();

    if length != 0.0 {
        let normal = (
            delta.1 / length * line_thickness,
            -delta.0 / length * line_thickness,
        );

        let points = vec![
            drawing::Point::new(
                (from.0 - normal.0 / 2.0).floor() as i32,
                (from.1 - normal.1 / 2.0).floor() as i32,
            ),
            drawing::Point::new(
                (to.0 - normal.0 / 2.0).floor() as i32,
                (to.1 - normal.1 / 2.0).floor() as i32,
            ),
            drawing::Point::new(
                (to.0 + normal.0 / 2.0).floor() as i32,
                (to.1 + normal.1 / 2.0).floor() as i32,
            ),
            drawing::Point::new(
                (from.0 + normal.0 / 2.0).floor() as i32,
                (from.1 + normal.1 / 2.0).floor() as i32,
            ),
        ];

        drawing::draw_convex_polygon_mut(image, &points, ::image::Rgba(color))
    }
}

/// Gets the used space in the image
fn get_used(image: &::image::RgbaImage) -> (u32, u32, u32, u32) {
    let mut min = (image.width(), image.height());
    let mut max = (0, 0);

    let blank = ::image::Rgba([0, 0, 0, 0]);
    for x in 0..image.width() {
        for y in 0..image.height() {
            let pixel = *image.get_pixel(x, y);
            if pixel != blank {
                min.0 = x.min(min.0);
                min.1 = y.min(min.1);
                max.0 = x.max(max.0);
                max.1 = y.max(max.1);
            }
        }
    }

    (min.0, min.1, max.0, max.1)
}

fn save(image: &::image::RgbaImage) {
    let (start_x, start_y, end_x, end_y) = get_used(image);

    let width = end_x - start_x;
    let height = end_y - start_y;

    // Remove all the blank areas
    let cropped_image =
        ::image::imageops::crop(&mut image.clone(), start_x, start_y, width + 1, height + 1)
            .to_image();
    cropped_image.save("save.png").unwrap();
}
