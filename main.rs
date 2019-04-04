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
    let mut draw_image =
        ::image::RgbaImage::new(window.size().width as u32, window.size().height as u32);
    let mut draw_texture = Texture::from_image(
        &mut window.factory,
        &draw_image,
        &TextureSettings::new().filter(Filter::Nearest).mag(Filter::Nearest),
    )
    .unwrap();

    let mut new_trace = vec![(0.0, 0.0)];
    let mut positions = Vec::new();
    let mut time = 0.0;
    let mut speed = 1;
    let mut running = true;
    while let Some(e) = window.next() {
        gui.event(&e);

        if let Some(dimensions) = e.resize_args() {
            draw_image = ::image::RgbaImage::new(dimensions[0] as u32, dimensions[1] as u32);
            draw_texture = Texture::from_image(
                &mut window.factory,
                &draw_image,
                &TextureSettings::new().filter(Filter::Nearest),
            )
            .unwrap();
        }

        let offset_x = f64::from(draw_image.width()) / 2.0;
        let offset_y = f64::from(draw_image.height()) / 2.0;
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
                let a = (a.0 + offset_x + 0.5, a.1 + offset_y + 0.5);
                let b = (b.0 + offset_x + 0.5, b.1 + offset_y + 0.5);
                //let a = (0.0, 0.0);
                //let b = (50.0, 25.0);
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
                image(&draw_texture, c.transform, g);

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
                    for pixel in draw_image.pixels_mut() {
                        *pixel = ::image::Rgba([0, 0, 0, 0]);
                    }
                }

                for (i, arm_input) in &mut arms_input.iter().enumerate() {
                    let (mut length, mut velocity) = world.arms[i];

                    arm_input.input(0, &mut length);
                    arm_input.input(1, &mut velocity);

                    world.arms[i] = (length, velocity);
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
                }
                if remove_button.is_pressed() {
                    arms_input.pop();
                    world.arms.pop();
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

fn ipart(x: f64) -> f64 {
    x.floor()
}

fn round(x: f64) -> f64 {
    ipart(x + 0.5)
}

fn fpart(x: f64) -> f64 {
    x - ipart(x)
}

fn rfpart(x: f64) -> f64 {
    1.0 - fpart(x)
}

fn draw_point(image: &mut ::image::RgbaImage, x: f64, y: f64, c: f64) {
    let alpha = (c * 255.0).min(255.0).max(0.0) as u8;
    let color = [255, 0, 0, alpha];
    image.put_pixel(x as u32, y as u32, ::image::Rgba(color));
}

fn draw_line(image: &mut ::image::RgbaImage, from: &(f64, f64), to: &(f64, f64), color: [u8; 4]) {
    let (mut x1, mut y1) = from;
    let (mut x2, mut y2) = to;

    let mut dx = x2 - x1;
    let mut dy = y2 - y1;
    let ax = dx.abs();
    let ay = dy.abs();

    let mut plot: Box<FnMut(f64, f64, f64)>;
    if ax < ay {
        std::mem::swap(&mut x1, &mut y1);
        std::mem::swap(&mut x2, &mut y2);
        std::mem::swap(&mut dx, &mut dy);
        plot = Box::new(|x, y, c| {
            draw_point(image, y, x, c);
        });
    } else {
        plot = Box::new(|x, y, c| {
            draw_point(image, x, y, c);
        });
    }
    if x2 < x1 {
        std::mem::swap(&mut x1, &mut x2);
        std::mem::swap(&mut y1, &mut y2);
    }
    let gradient = dy / dx;

    let xend = round(x1);
    let yend = y1 + gradient * (xend - x1);
    let xgap = rfpart(x1 + 0.5);
    let xpxl1 = xend;
    let ypxl1 = ipart(yend);
    plot(xpxl1, ypxl1, rfpart(yend) * xgap);
    plot(xpxl1, ypxl1 + 1.0, fpart(yend) * xgap);
    let mut intery = yend + gradient;

    let xend = round(x2);
    let yend = y2 + gradient * (xend - x2);
    let xgap = fpart(x2 + 0.5);
    let xpxl2 = xend;
    let ypxl2 = ipart(yend);
    plot(xpxl2, ypxl2, rfpart(yend) * xgap);
    plot(xpxl2, ypxl2 + 1.0, fpart(yend) * xgap);

    for x in (xpxl1 as u32 + 1)..(xpxl2 as u32 - 1) {
        plot(x as f64, ipart(intery), rfpart(intery));
        plot(x as f64, ipart(intery), rfpart(intery));
        intery += gradient;
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
