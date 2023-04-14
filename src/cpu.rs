use core::time;

use sdl2::{pixels, rect::Rect, render::Canvas, video::Window};

use crate::font::FONT_SET;

pub const INSTR_SIZE: u16 = 2;
pub const ROWS: usize = 32;
pub const COLS: usize = 64;
pub struct CPU {
    v_reg: [u8; 16],
    i_reg: u16,
    sp: u8,
    pc: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    mem: [u8; 0x1000],
    screen: [[u8; COLS]; ROWS],
    canvas: Canvas<Window>,
}

impl CPU {
    pub fn new(rom: &str) -> Self {
        let mut mem = [0; 0x1000];

        for i in 0..FONT_SET.len() {
            mem[i] = FONT_SET[i];
        }

        let bytes = std::fs::read(rom).expect("Reading from ROM failed");
        for i in 0..bytes.len() {
            mem[(0x200 + i) as usize] = bytes[i as usize];
        }

        let sdl_context = sdl2::init().unwrap();
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys
            .window("xxx", (COLS * 20) as u32, (ROWS * 20) as u32)
            .position_centered()
            .opengl()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Self {
            v_reg: [0; 16],
            i_reg: 0,
            sp: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            mem,
            screen: [[0; COLS]; ROWS],
            canvas,
        }
    }
}

impl CPU {
    fn fetch(&mut self) -> u16 {
        let hi: u16 = self.mem[self.pc as usize] as u16;
        let lo: u16 = self.mem[(self.pc + 1) as usize] as u16;
        let res: u16 = (hi << 8) | lo;
        self.pc += INSTR_SIZE;
        res
    }

    fn print_reg(&self) {
        for i in 0..16 {
            print!("{:04X},", self.v_reg[i]);
        }
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.fetch();
            // self.pc += INSTR_SIZE;
            self.step(opcode);
            self.draw_to_screen();
            print!("{:04X},", opcode);
            self.print_reg();
            print!("{:04X}\n", self.i_reg);
            if opcode == 5084 {
                break;
            }
            std::thread::sleep(time::Duration::from_millis(10));
        }
    }

    pub fn step(&mut self, opcode: u16) {
        let nnn = Self::nnn(opcode);
        let kk = Self::kk(opcode);
        match Self::nibbles(opcode) {
            (0x0, 0x0, 0xE, 0x0) => todo!("CLS"),
            (0x0, 0x0, 0xE, 0xE) => self.ret_00ee(),
            (0x1, _, _, _) => self.jp_1nnn(nnn),
            (0x2, _, _, _) => self.call_2nnn(nnn),
            (0x3, x, _, _) => self.se_3xkk(x, kk),
            (0x4, x, _, _) => self.sne_4kk(x, kk),
            (0x5, x, y, _) => self.se_5xy0(x, y),
            (0x6, x, _, _) => self.ld_6xkk(x, kk),
            (0x7, x, _, _) => self.add_7xkk(x, kk),
            (0x8, x, y, 0x0) => self.ld_8xy0(x, y),
            (0x8, x, y, 0x1) => self.or_8xy1(x, y),
            (0x8, x, y, 0x2) => self.and_8xy2(x, y),
            (0x8, x, y, 0x3) => self.xor_8xy3(x, y),
            (0x8, x, y, 0x4) => self.add_8xy4(x, y),
            (0x8, x, y, 0x5) => self.sub_8xy5(x, y),
            (0x8, x, y, 0x6) => self.shr_8xy6(x, y),
            (0x8, x, y, 0x7) => self.subn_8xy7(x, y),
            (0x8, x, _, 0xE) => self.shl_8xye(x),
            (0x9, x, y, 0x0) => self.sne_9xy0(x, y),
            (0xA, _, _, _) => self.ldi_annn(nnn),
            (0xB, _, _, _) => self.jpv0_bnnn(nnn),
            (0xC, x, _, _) => self.rnd_cxkk(x, kk),
            (0xD, x, y, n) => self.drw_dxyn(x, y, n),
            (0xE, x, 0x9, 0xE) => self.skp_ex9e(x),
            (0xE, x, 0xA, 0x1) => self.sknp_exa1(x),
            (0xF, x, 0x0, 0x7) => self.ld_fx07(x),
            (0xF, x, 0x0, 0xA) => self.ld_fx0a(x),
            (0xF, x, 0x1, 0x5) => self.ld_fx15(x),
            (0xF, x, 0x1, 0x8) => self.ld_fx18(x),
            (0xF, x, 0x1, 0xE) => self.add_fx1e(x),
            (0xF, x, 0x2, 0x9) => self.ld_fx29(x),
            (0xF, x, 0x3, 0x3) => self.ld_fx33(x),
            (0xF, x, 0x5, 0x5) => self.ld_fx55(x),
            (0xF, x, 0x6, 0x5) => self.ld_fx65(x),
            (_, _, _, _) => panic!("Tried to process a bad opcode: {opcode}"),
        }
    }

    fn ret_00ee(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
    }

    fn jp_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn call_2nnn(&mut self, nnn: u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = nnn;
    }

    fn se_3xkk(&mut self, x: u8, kk: u8) {
        if self.reg(x) == kk {
            self.pc += INSTR_SIZE;
        }
    }

    fn sne_4kk(&mut self, x: u8, kk: u8) {
        if self.reg(x) != kk {
            self.pc += INSTR_SIZE;
        }
    }

    fn se_5xy0(&mut self, x: u8, y: u8) {
        if self.reg(x) == self.reg(y) {
            self.pc += INSTR_SIZE;
        }
    }

    fn ld_6xkk(&mut self, x: u8, kk: u8) {
        self.set_reg(x, kk);
    }

    fn add_7xkk(&mut self, x: u8, kk: u8) {
        self.set_reg(x, self.reg(x).wrapping_add(kk));
    }

    fn ld_8xy0(&mut self, x: u8, y: u8) {
        self.set_reg(x, self.reg(y));
    }

    fn or_8xy1(&mut self, x: u8, y: u8) {
        self.set_reg(x, self.reg(x) | self.reg(y));
    }

    fn and_8xy2(&mut self, x: u8, y: u8) {
        self.set_reg(x, self.reg(x) & self.reg(y));
    }

    fn xor_8xy3(&mut self, x: u8, y: u8) {
        self.set_reg(x, self.reg(x) ^ self.reg(y));
    }

    fn add_8xy4(&mut self, x: u8, y: u8) {
        // self.print_reg();
        // print!("\n");
        let vx = self.v_reg[x as usize] as u16;
        let vy = self.v_reg[y as usize] as u16;
        let res = vx + vy;
        let vf = if res > 0xFF { 1 } else { 0 };
        // println!("{x}");
        // println!("{y}");
        // println!("{vx}");
        // println!("{vy}");
        // println!("{res}");
        self.set_reg(0xF, vf);
        self.set_reg(x, res as u8);
    }

    fn sub_8xy5(&mut self, x: u8, y: u8) {
        let vf = if self.reg(y) > self.reg(x) { 0 } else { 1 };

        self.set_reg(0xF, vf);
        self.set_reg(x, self.reg(x).wrapping_sub(self.reg(y)));
    }

    fn shr_8xy6(&mut self, x: u8, _y: u8) {
        let lsb = self.reg(x) & 0x01;
        self.set_reg(0x0F, lsb);
        self.set_reg(x, self.reg(x) >> 1);
    }

    fn subn_8xy7(&mut self, x: u8, y: u8) {
        let vf = if self.reg(y) > self.reg(x) { 0 } else { 1 };

        self.set_reg(0xF, vf);
        self.set_reg(x, self.reg(y).wrapping_sub(self.reg(x)));
    }

    fn shl_8xye(&mut self, x: u8) {
        let vf = if self.reg(x) & 0b1000_0000 == 0b1000_0000 {
            1
        } else {
            0
        };

        self.set_reg(0xF, vf);
        self.set_reg(x, self.reg(x) << 1);
    }

    fn sne_9xy0(&mut self, x: u8, y: u8) {
        if self.reg(x) != self.reg(y) {
            self.pc += INSTR_SIZE
        }
    }

    fn ldi_annn(&mut self, nnn: u16) {
        self.i_reg = nnn;
    }

    fn jpv0_bnnn(&mut self, nnn: u16) {
        self.pc = (self.reg(0x00) as u16) + nnn;
    }

    fn rnd_cxkk(&mut self, x: u8, kk: u8) {
        self.set_reg(x, rand::random::<u8>() & kk)
    }

    fn drw_dxyn(&mut self, mut x: u8, y: u8, n: u8) {
        let mut collision = false;
        for i in 0..n {
            let addr: usize = (self.i_reg + i as u16).into();
            let b = self.mem[addr];
            if b & self.screen[y as usize][x as usize] == 0x01 {
                collision = true;
            }
            self.screen[y as usize][x as usize] = b;
            x = (x + 1) % (COLS as u8);
        }

        self.set_reg(0xF, if collision { 0x1 } else { 0x0 })
    }

    fn skp_ex9e(&mut self, _x: u8) {}

    fn sknp_exa1(&mut self, _x: u8) {}

    fn ld_fx07(&mut self, x: u8) {
        self.set_reg(x, self.delay_timer);
    }

    fn ld_fx0a(&mut self, _x: u8) {}

    fn ld_fx15(&mut self, x: u8) {
        self.delay_timer = self.reg(x);
    }

    fn ld_fx18(&mut self, x: u8) {
        self.sound_timer = self.reg(x);
    }

    fn add_fx1e(&mut self, x: u8) {
        self.i_reg += self.reg(x) as u16;
    }

    fn ld_fx29(&mut self, x: u8) {
        self.i_reg = (5 * self.reg(x)).into();
    }

    fn ld_fx33(&mut self, x: u8) {
        let dec = self.reg(x);

        let ones = dec % 10;
        let tens = (dec / 10) % 10;
        let hundreds = (dec / 100) % 10;

        self.mem[self.i_reg as usize] = hundreds;
        self.mem[(self.i_reg + 1) as usize] = tens;
        self.mem[(self.i_reg + 2) as usize] = ones;
    }

    fn ld_fx55(&mut self, x: u8) {
        for i in 0..=x {
            let addr: usize = (self.i_reg + (i as u16)).into();
            self.mem[addr] = self.reg(x);
        }

        self.i_reg = self.i_reg + x as u16 + 1;
    }

    fn ld_fx65(&mut self, x: u8) {
        for i in 0..=x {
            let addr: usize = (self.i_reg + (i as u16)).into();
            self.set_reg(i, self.mem[addr])
        }

        self.i_reg = self.i_reg + x as u16 + 1;
    }
}

impl CPU {
    fn nibbles(opcode: u16) -> (u8, u8, u8, u8) {
        let a = ((opcode & 0xF000) >> 12) as u8;
        let b = ((opcode & 0x0F00) >> 8) as u8;
        let c = ((opcode & 0x00F0) >> 4) as u8;
        let d = (opcode & 0x000F) as u8;
        (a, b, c, d)
    }

    fn nnn(opcode: u16) -> u16 {
        opcode & 0x0FFF
    }

    fn kk(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }

    fn reg(&self, x: u8) -> u8 {
        self.v_reg[x as usize]
    }

    fn set_reg(&mut self, x: u8, val: u8) {
        self.v_reg[x as usize] = val;
    }
}

impl CPU {
    fn draw_to_screen(&mut self) {
        let scale_factor = 20;
        for (y, row) in self.screen.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                let x = (x as u32) * scale_factor;
                let y = (y as u32) * scale_factor;
                self.canvas.set_draw_color(Self::color(col));
                let _ = self.canvas.fill_rect(Rect::new(
                    x as i32,
                    y as i32,
                    scale_factor,
                    scale_factor,
                ));
            }
        }
        self.canvas.present();
    }

    fn color(val: u8) -> pixels::Color {
        if val == 0 {
            pixels::Color::RGB(0, 0, 0)
        } else {
            pixels::Color::RGB(0, 250, 0)
        }
    }
}
