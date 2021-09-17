use rnes::{cpu, roms};

fn main() {
    //let x = 0xabcd;
    //println!("{:?}", utils::split_u16(x));

    //println!("{}", 227 & 128 == 128);

    run_official_only();
    //sdl2_test().unwrap();
}

fn run_official_only() {
    //let path = "nes_test_roms\\instr_test-v3\\official_only.nes";
    //let path = "nes_test_roms\\instr_test-v3\\rom_singles\\01-implied.nes";
    //let path = "nes_test_roms\\instr_test-v3\\rom_singles\\02-immediate.nes";

    //let path = "nes_test_roms\\instr_misc\\instr_misc.nes";
    //let path = "nes_test_roms\\blargg_nes_cpu_test5\\official.nes";
    let path = "nestest.nes";

    let cartridge = roms::read_rom(path);
    let mut cpu = cpu::Cpu::new();
    cpu.load_cartridge(cartridge);
    cpu.init();

    cpu.set_pc(0xC000);
    cpu.logger.enable_logging();

    let mut i = 0;

    loop {
        cpu.tick();
        #[cfg(debug_assertions)]
        {
            println!("{:?}", &cpu);
            //println!("0x6000: {}", cpu.peek(0x6000));
            //let mut buffer = vec![];
            //let mut j = 0x6004;
            //let mut b = cpu.peek(j);
            //for _ in 0..1000 {
                //j += 1;
                //buffer.push(b);
                //b = cpu.peek(j);
            //}
            //let s = String::from_utf8(buffer).unwrap();
            //println!("{}", s);
        }

        //if cpu.get_cycles() > 3600 {
            //for i in 0..0xFF {
                //let a: u16 = 0x0700 + i;
                //println!("{:#06x}: {:#04x}", a, cpu.peek(a));
            //}
            //panic!();
        //}
        //if i == 220000 {
            //panic!();
        //}

        i += 1;
        if cpu.logger.is_logging() {
            if i == 27000 {
                println!("{:?}", &cpu);
                cpu.logger.print_log();
                cpu.logger.write_log("out.txt");
                panic!();
            }
        }

        if i == 1000000 {
            println!("cycles: {}", cpu.get_cycles());
            println!("0x6000: {}", cpu.peek(0x6000));
            let mut buffer = vec![];
            let mut j = 0x6004;
            let mut b = cpu.peek(j);
            //while b != 0 {
            for _ in 0..1000 {
                j += 1;
                buffer.push(b);
                b = cpu.peek(j);
            }
            let s = String::from_utf8_lossy(&buffer);
            println!("{}", s);
            i = 0;
            //std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

static SCREEN_WIDTH: u32 = 800;
static SCREEN_HEIGHT: u32 = 600;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
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

fn sdl2_test() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let font = ttf_context.load_font("00antix.ttf", 22).unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
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
        let surface = font
            .render("Hello Rust!")
            .blended(Color::RGBA(255, 255, 255, 255))
            .map_err(|e| e.to_string())?;
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;
        let TextureQuery { width, height, .. } = texture.query();

        let padding = 64;
        let target = get_centered_rect(
            width,
            height,
            SCREEN_WIDTH - padding,
            SCREEN_HEIGHT - padding,
        );

        canvas.copy(&texture, None, Some(target))?;

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
