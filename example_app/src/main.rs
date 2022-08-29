const TICK_IN_MS : u128 = 250;

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, DeviceEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use winit::event::KeyboardInput;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use code_editor::prelude::*;

/// Gets the current time in milliseconds
fn get_time() -> u128 {
    let stop = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
        stop.as_millis()
}

fn main() -> Result<(), Error> {

    let mut width     : usize = 600;
    let mut height    : usize = 400;

    env_logger::init();

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(width as f64, height as f64);

        WindowBuilder::new()
            .with_title("CodeEditor")
            .with_inner_size(size)
            .with_min_inner_size(size)

            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(width as u32, height as u32, surface_texture)?
    };

    // Init the code editor

    let mut code_editor = CodeEditor::new();
    code_editor.set_font("fonts/Source_Code_Pro/static/SourceCodePro-Regular.ttf");
    code_editor.set_mode(CodeEditorMode::Rhai);
    code_editor.set_font_size(17.0);

    // Taken from https://github.com/rhaiscript/rhai/blob/main/scripts/fibonacci.rhai
    code_editor.set_text(
r#"//! This script calculates the n-th Fibonacci number using a really dumb algorithm
//! to test the speed of the scripting engine.

const TARGET = 28;
const REPEAT = 5;
const ANSWER = 317811;

fn fib(n) {
    if n < 2 {
        n
    } else {
        fib(n-1) + fib(n-2)
    }
}

print(`Running Fibonacci(${TARGET}) x ${REPEAT} times...`);
print("Ready... Go!");

let result;
let now = timestamp();

for n in 0..REPEAT {
    result = fib(TARGET);
}

print(`Finished. Run time = ${now.elapsed} seconds.`);

print(`Fibonacci number #${TARGET} = ${result}`);

if result != ANSWER {
    print(`The answer is WRONG! Should be ${ANSWER}!`);
}
    "#.to_string());

    let mut timer : u128 = 0;

    event_loop.run(move |event, _, control_flow| {
        use winit::event::{ElementState, VirtualKeyCode};

        if let Event::RedrawRequested(_) = event {

            let frame = pixels.get_frame();
            code_editor.draw(frame, (0, 0, width, height), width);

            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        match &event {

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::ReceivedCharacter(char ) => match char {
                    _ => {
                        if code_editor.key_down(Some(*char), None) {
                            window.request_redraw();
                        }
                    }
                },

                WindowEvent::ModifiersChanged(state) => match state {
                    _ => {
                        if code_editor.modifier_changed(state.shift(), state.ctrl(), state.alt(), state.logo()) {
                            window.request_redraw();
                        }
                    }
                },

                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(virtual_code),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match virtual_code {
                    VirtualKeyCode::Delete => {
                        if code_editor.key_down(None, Some(WidgetKey::Delete)) {
                            window.request_redraw();
                        }
                    },
                    VirtualKeyCode::Back => {
                        if code_editor.key_down(None, Some(WidgetKey::Delete)) {
                            window.request_redraw();
                        }
                    },
                    VirtualKeyCode::Up => {
                        if code_editor.key_down(None, Some(WidgetKey::Up)) {
                            window.request_redraw();
                        }
                    },
                    VirtualKeyCode::Right => {
                        if code_editor.key_down(None, Some(WidgetKey::Right)) {
                            window.request_redraw();
                        }
                    },
                    VirtualKeyCode::Down => {
                        if code_editor.key_down(None, Some(WidgetKey::Down)) {
                            window.request_redraw();
                        }
                    },
                    VirtualKeyCode::Left => {
                        if code_editor.key_down(None, Some(WidgetKey::Left)) {
                            window.request_redraw();
                        }
                    },
                    VirtualKeyCode::Space => {
                        if code_editor.key_down(None, Some(WidgetKey::Space)) {
                            window.request_redraw();
                        }
                    },
                    VirtualKeyCode::Tab => {
                        if code_editor.key_down(None, Some(WidgetKey::Tab)) {
                            window.request_redraw();
                        }
                    },
                    VirtualKeyCode::Return => {
                        if code_editor.key_down(None, Some(WidgetKey::Return)) {
                            window.request_redraw();
                        }
                    },
                    VirtualKeyCode::Escape => {
                        if code_editor.key_down(None, Some(WidgetKey::Escape)) {
                            window.request_redraw();
                        }
                    }
                    _ => (),
                },
                _ => (),
            },

            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::Text { codepoint } => {
                    println!("text: ({})", codepoint);
                }
                DeviceEvent::MouseWheel { delta } => match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        println!("mouse wheel Line Delta: ({},{})", x, y);
                    }
                    winit::event::MouseScrollDelta::PixelDelta(p) => {
                        //println!("mouse wheel Pixel Delta: ({},{})", p.x, p.y);
                        if code_editor.mouse_wheel((p.x as isize, p.y as isize)) {
                            window.request_redraw();
                            //mouse_wheel_ongoing = true;
                        }

                        if p.x == 0.0 && p.y == 0.0 {
                            //mouse_wheel_ongoing = false;
                        }
                    }
                },
                _ => (),
            },
            _ => (),
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if /*input.key_pressed(VirtualKeyCode::Escape) ||*/ input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if input.mouse_pressed(0) {
                let coords =  input.mouse().unwrap();
                let pixel_pos: (usize, usize) = pixels.window_pos_to_pixel(coords)
                   .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                if code_editor.mouse_down(pixel_pos) {
                    window.request_redraw();
                }
            }

            if input.mouse_released(0) {
                let coords =  input.mouse().unwrap();
                let pixel_pos: (usize, usize) = pixels.window_pos_to_pixel(coords)
                   .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                if code_editor.mouse_up(pixel_pos) {
                    window.request_redraw();
                }
            }

            if input.mouse_held(0) {
                let diff =  input.mouse_diff();
                if diff.0 != 0.0 || diff.1 != 0.0 {
                    let coords =  input.mouse().unwrap();
                    let pixel_pos: (usize, usize) = pixels.window_pos_to_pixel(coords)
                       .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                    if code_editor.mouse_dragged(pixel_pos) {
                        window.request_redraw();
                    }
                }
            } else {
                let diff =  input.mouse_diff();
                if diff.0 != 0.0 || diff.1 != 0.0 {
                    let coords =  input.mouse().unwrap();
                    let pixel_pos: (usize, usize) = pixels.window_pos_to_pixel(coords)
                       .unwrap_or_else(|pos| pixels.clamp_pixel_pos(pos));

                    if code_editor.mouse_hover(pixel_pos) {
                        window.request_redraw();
                    }
                }
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                let scale = window.scale_factor() as u32;
                pixels.resize_buffer(size.width / scale, size.height / scale);
                width = size.width as usize / scale as usize;
                height = size.height as usize / scale as usize;
                window.request_redraw();
            }

            let curr_time = get_time();

            // We update the screen 4 times a second, change TICK_IN_MS to change the refresh rate

            if curr_time > timer + TICK_IN_MS {
                window.request_redraw();
                timer = curr_time;
            } else {
                let t = (timer + TICK_IN_MS - curr_time) as u64;
                if t > 10 {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
    });
}
