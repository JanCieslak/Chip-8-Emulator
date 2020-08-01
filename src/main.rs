use minifb::{WindowOptions, Window, Key};
use crate::emu::chip8;
use crate::emu::chip8::Chip8;

mod emu;

fn main() {
    let mut chip8 = Chip8::new();
    let mut window = Window::new(
        "Chip8 Emulator",
        512,
        256,
        WindowOptions::default()
    ).unwrap();

    // limit to 60 fps
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if chip8.draw_flag {
            chip8.draw_flag = false;
            window.update_with_buffer(&chip8.gfx, 64, 32).unwrap();
        }

        window.update();
    }
}
