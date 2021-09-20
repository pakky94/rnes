#![allow(dead_code)]
use rnes::cartridge::Cartridge;
use rnes::roms;

fn main() {
    run_emulator().unwrap();
}

static SCREEN_WIDTH: u32 = 800;
static SCREEN_HEIGHT: u32 = 600;

extern crate sdl2;

use sdl2::event::Event;
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
    let path = "Donkey Kong.nes";
    //let path = "bomberman.nes";
    //let path = "Metroid.nes";
    //let path = "nestest.nes";
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

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();

    let cartridge = get_cartridge();
    let mut nes = rnes::Nes::with_cartridge(cartridge);
    
    //nes.enable_logging();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {55;
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
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
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
