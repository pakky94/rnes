#![allow(dead_code)]
use rnes::cartridge::Cartridge;
use rnes::input::InputData;
use rnes::roms;

fn main() {
    run_emulator().unwrap();
}

static SCREEN_WIDTH: u32 = 1024;
static SCREEN_HEIGHT: u32 = 768;

extern crate sdl2;

use sdl2::controller::{Button, GameController};
use sdl2::event::Event;
use sdl2::joystick::HatState;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use std::time::Duration;

// handle the annoying Rect i32
macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

// Scale fonts to a reasonable size when they're too big (though they might look less smooth)
fn get_centered_rect(rect_width: u32, rect_height: u32, cons_width: u32, cons_height: u32) -> Rect {
    let wr = rect_width as f32 / cons_width as f32;
    let hr = rect_height as f32 / cons_height as f32;

    let (w, h) = if wr > 1f32 || hr > 1f32 {
        if wr > hr {
            println!("Scaling down! The text will look worse!");
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            println!("Scaling down! The text will look worse!");
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };

    let cx = (SCREEN_WIDTH as i32 - w) / 2;
    let cy = (SCREEN_HEIGHT as i32 - h) / 2;
    rect!(cx, cy, w, h)
}

fn get_cartridge() -> Cartridge {
    //let path = "Super Mario Bros.nes";
    //let path = "Donkey Kong.nes";
    //let path = "bomberman.nes";
    //let path = "Metroid.nes";
    let path = "zelda.nes";
    //let path = "nestest.nes";
    //let path = "nes_test_roms/instr_test-v3/all_instrs.nes";
    //let path = "nes_test_roms/cpu_interrupts_v2/cpu_interrupts.nes";
    let cartridge = roms::read_rom(path);
    cartridge
}

fn run_emulator() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    // Timing stuff
    let mut target_time =
        time::Instant::now() + (time::Duration::nanoseconds(1_000_000_000i64 / 60));

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();

    let cartridge = get_cartridge();
    let mut nes = rnes::Nes::with_cartridge(cartridge);

    //nes.enable_logging();

    #[cfg(debug_assertions)]
    nes.enable_logging();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let ctrl_sys = sdl_context.game_controller().unwrap();

    let controller: Option<GameController> = match ctrl_sys.open(0) {
        Ok(controller) => Some(controller),
        Err(_) => None,
    };

    let mut inputs = InputData {
        a: false,
        b: false,
        up: false,
        down: false,
        left: false,
        right: false,
        start: false,
        select: false,
    };

    // non games buttons
    let mut log_pressed = false;

    let mut palette_idx = 0;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        let was_logging = log_pressed;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::Up) => inputs.up = true,
                    Some(Keycode::Down) => inputs.down = true,
                    Some(Keycode::Left) => inputs.left = true,
                    Some(Keycode::Right) => inputs.right = true,
                    Some(Keycode::S) => inputs.start = true,
                    Some(Keycode::A) => inputs.select = true,
                    Some(Keycode::Z) => inputs.b = true,
                    Some(Keycode::X) => inputs.a = true,

                    Some(Keycode::P) => palette_idx = (palette_idx + 1) % 8,
                    Some(Keycode::L) => log_pressed = true,
                    _ => {}
                },
                Event::KeyUp { keycode, .. } => match keycode {
                    Some(Keycode::Up) => inputs.up = false,
                    Some(Keycode::Down) => inputs.down = false,
                    Some(Keycode::Left) => inputs.left = false,
                    Some(Keycode::Right) => inputs.right = false,
                    Some(Keycode::S) => inputs.start = false,
                    Some(Keycode::A) => inputs.select = false,
                    Some(Keycode::Z) => inputs.b = false,
                    Some(Keycode::X) => inputs.a = false,

                    Some(Keycode::L) => log_pressed = false,
                    _ => {}
                },
                Event::JoyButtonDown { button_idx, .. } => handle_button_press(&mut inputs, button_idx),
                Event::JoyButtonUp { button_idx, .. } => handle_button_release(&mut inputs, button_idx),
                //Event::JoyAxisMotion { axis_idx, value, .. } => panic!("idx: {}, val: {}", axis_idx, value),
                Event::JoyHatMotion { state, .. } => handle_dpad(&mut inputs, state),
                _ => {}
            }
        }

        nes.set_input1(inputs.clone());

        if log_pressed {
            nes.enable_logging();
        } else if was_logging {
            nes.write_cpu_logs("log.log");
            nes.disable_logging();
            nes.clear_logs();
        }

        // The rest of the game loop goes here...
        nes.run_until_frame();
        let mut frame = nes.get_frame();
        let game_render = Surface::from_data(
            frame.get_data(),
            256,
            224,
            256 * 4,
            sdl2::pixels::PixelFormatEnum::RGBA8888,
        )?
        .as_texture(&texture_creator)
        .map_err(|e| e.to_string())?;

        canvas.copy(&game_render, None, Some(Rect::new(0, 0, 512, 448)))?;
        nes.return_frame(frame);

        if true {
            let mut palette0 = nes.render_pattern_table(0x0, palette_idx);
            let palette0_render = Surface::from_data(
                palette0.get_data(),
                128,
                128,
                128 * 4,
                sdl2::pixels::PixelFormatEnum::RGBA8888,
            )?
                .as_texture(&texture_creator)
                .map_err(|e| e.to_string())?;

            canvas.copy(&palette0_render, None, Some(Rect::new(0, 460, 256, 256)))?;

            let mut palette1 = nes.render_pattern_table(0x1000, palette_idx);
            let palette1_render = Surface::from_data(
                palette1.get_data(),
                128,
                128,
                128 * 4,
                sdl2::pixels::PixelFormatEnum::RGBA8888,
            )?
                .as_texture(&texture_creator)
                .map_err(|e| e.to_string())?;
            
            canvas.copy(&palette1_render, None, Some(Rect::new(256, 460, 256, 256)))?;

            for i in 0..4 {
                let mut nametable = nes.render_nametable(i);
                let nametable_render = Surface::from_data(
                    nametable.get_data(),
                    256,
                    240,
                    256 * 4,
                    sdl2::pixels::PixelFormatEnum::RGBA8888,
                )?
                    .as_texture(&texture_creator)
                    .map_err(|e| e.to_string())?;

                let (x, y) = match i {
                    0 => (0, 0),
                    1 => (256, 0),
                    2 => (0, 240),
                    3 => (256, 240),
                    _ => unreachable!(),
                };
                let x = x + 512;
                
                canvas.copy(&nametable_render, None, Some(Rect::new(x, y, 256, 240)))?;
            }
        }


        //let surface = font
        //.render("Hello Rust!")
        //.blended(Color::RGBA(255, 255, 255, 255))
        //.map_err(|e| e.to_string())?;
        //let texture = texture_creator
        //.create_texture_from_surface(&surface)
        //.map_err(|e| e.to_string())?;
        //let TextureQuery { width, height, .. } = texture.query();

        //let padding = 64;
        //let target = get_centered_rect(
        //width,
        //height,
        //SCREEN_WIDTH - padding,
        //SCREEN_HEIGHT - padding,
        //);
        //
        //canvas.copy(&texture, None, Some(target))?;

        canvas.present();
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        let time_now = time::Instant::now();
        let sleep_time =  std::cmp::max(0, (target_time - time_now).subsec_nanoseconds());
        target_time = time_now + time::Duration::nanoseconds(1_000_000_000i64 / 60);
        ::std::thread::sleep(Duration::new(0, sleep_time as u32));
    }

    Ok(())
}

fn handle_button_press(inputs: &mut InputData, button_idx: u8) {
    match button_idx {
        0 => inputs.b = true,
        1 => inputs.a = true,
        6 => inputs.select = true,
        7 => inputs.start = true,
        _ => {},
        //_ => todo!("{}", button_idx),
    }
}

fn handle_button_release(inputs: &mut InputData, button_idx: u8) {
    match button_idx {
        0 => inputs.b = false,
        1 => inputs.a = false,
        6 => inputs.select = false,
        7 => inputs.start = false,
        _ => {},
        //_ => todo!(),
    }
}

fn handle_dpad(inputs: &mut InputData, state: HatState) {
    match state {
        HatState::Centered => {
            inputs.up = false;
            inputs.down = false;
            inputs.left = false;
            inputs.right = false;
        }
        HatState::Up => inputs.up = true,
        HatState::Right => inputs.right = true,
        HatState::Down => inputs.down = true,
        HatState::Left => inputs.left = true,
        HatState::RightUp => {
            inputs.right = true;
            inputs.up = true;
        }
        HatState::RightDown => {
            inputs.right = true;
            inputs.down = true;
        }
        HatState::LeftUp => {
            inputs.left = true;
            inputs.up = true;
        }
        HatState::LeftDown => {
            inputs.left = true;
            inputs.down = true;
        }
    }
}
