extern crate sdl2;

pub struct Video {
    pub lcdc: u8,
    pub stat: u8,
    pub ly: u8,
    counter: u32,

    pub vram: [u8; 0x2000],
    pub oam: [u8; 0x100],
    pub vblank_interrupt: bool,

    canvas: sdl2::render::WindowCanvas,
    event_pump: sdl2::EventPump,
}

impl Video {
    pub fn new() -> Video {
        //sdl2::hint::set("SDL_HINT_RENDER_VSYNC", "1");

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("not-so-gb", 160, 144)
            .position_centered()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        return Video {
            lcdc: 0x80,
            stat: 0x00,
            ly: 0,
            counter: 0,

            vram: [0; 0x2000],
            oam: [0; 0x100],
            vblank_interrupt: false,

            canvas: canvas,
            event_pump: event_pump,
        };
    }

    pub fn step(&mut self) {
        self.counter += 1;
        self.stat &= !0x03;
        if self.ly >= 144 {
            self.stat |= 0x01;
        } else if self.counter < 10 {
            self.stat |= 0x02;
        } else if self.counter < 50 {
            self.stat |= 0x03;
        }
        if self.counter < 100 {
            return;
        }
        self.counter = 0;
        self.ly = self.ly.wrapping_add(1);
        if self.ly == 145 {
            self.vblank_interrupt = true;

            self.canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
            self.canvas.clear();

            for y in 0..144 {
                let bgy = y;
                let bg_tile_row = y % 8;
                for x in 0..160 {
                    let bgx = x;
                    let tile_number = self.vram[0x1800 + bgx / 8 + bgy / 8 * 0x20] as usize;
                    let tile_data_index = tile_number * 16 + bg_tile_row * 2;
                    let a = self.vram[tile_data_index + 0];
                    let b = self.vram[tile_data_index + 1];
                    let bit = 1 << (bgx % 8) as u8;
                    if (a & bit) == bit && (b & bit) == bit {
                        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
                    } else {
                        self.canvas.set_draw_color(sdl2::pixels::Color::RGB(128, 128, 128));
                    }
                    self.canvas
                        .draw_point(sdl2::rect::Point::new(x as i32, y as i32))
                        .unwrap();
                }
            }
            self.canvas.present();

            for event in self.event_pump.poll_iter() {
                match event {
                    sdl2::event::Event::Quit { .. }
                    | sdl2::event::Event::KeyDown {
                        keycode: Some(sdl2::keyboard::Keycode::Escape),
                        ..
                    } => panic!("QUIT"),
                    _ => {}
                }
            }
        }
    }
}
