#[derive(Copy, Clone, Debug)]
pub struct CPU {
    v_reg: [u8; 16],
    i_reg: u16,
    sp: u8,
    pc: u16,
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    mem: [u8; 0x1000],
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            v_reg: [0; 16],
            i_reg: 0,
            sp: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            mem: [0; 0x1000],
        }
    }
}

impl CPU {
    pub fn step(&mut self, opcode: u16) {
        let nnn = Self::nnn(opcode);
        let kk = Self::kk(opcode);
        match Self::nibbles(opcode) {
            (0x0, 0x0, 0xE, 0x0) => todo!("CLS"),
            (0x0, 0x0, 0xE, 0xE) => todo!("RET"),
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
            (0x8, x, y, 0xE) => self.shl_8xye(x, y),
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

    fn jp_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn call_2nnn(&mut self, nnn: u16) {
        unimplemented!("call")
    }

    fn se_3xkk(&mut self, x: u8, kk: u8) {
        if self.reg(x) == kk {
            self.pc += 2;
        }
    }

    fn sne_4kk(&mut self, x: u8, kk: u8) {
        if self.reg(x) != kk {
            self.pc += 2;
        }
    }

    fn se_5xy0(&mut self, x: u8, y: u8) {
        if self.reg(x) != self.reg(y) {
            self.pc += 2;
        }
    }

    fn ld_6xkk(&mut self, x: u8, kk: u8) {
        self.v_reg[x as usize] = kk;
    }

    fn add_7xkk(&mut self, x: u8, kk: u8) {
        self.v_reg[x as usize] += kk;
    }

    fn ld_8xy0(&mut self, x: u8, y: u8) {
        self.v_reg[x as usize] = self.v_reg[y as usize];
    }

    fn or_8xy1(&mut self, x: u8, y: u8) {
        self.v_reg[x as usize] |= self.v_reg[y as usize];
    }

    fn and_8xy2(&mut self, x: u8, y: u8) {
        self.v_reg[x as usize] &= self.v_reg[y as usize];
    }

    fn xor_8xy3(&mut self, x: u8, y: u8) {
        self.v_reg[x as usize] ^= self.v_reg[y as usize];
    }

    fn add_8xy4(&mut self, x: u8, y: u8) {
        self.v_reg[x as usize] -= self.v_reg[y as usize];
        unimplemented!("borrow flag")
    }

    fn sub_8xy5(&mut self, x: u8, y: u8) {
        self.v_reg[x as usize] -= self.v_reg[y as usize];
        unimplemented!("borrow flag")
    }

    fn shr_8xy6(&mut self, x: u8, y: u8) {
        let lsb = self.v_reg[x as usize] & 0x01;
        self.v_reg[0x0F] = lsb;
        // XXX: can we just shift
        self.v_reg[x as usize] /= 2;
    }

    fn subn_8xy7(&mut self, x: u8, y: u8) {
        todo!("subn8xy7")
    }

    fn shl_8xye(&mut self, x: u8, y: u8) {
        todo!()
    }

    fn sne_9xy0(&mut self, x: u8, y: u8) {
        todo!()
    }

    fn ldi_annn(&mut self, nnn: u16) {
        self.i_reg = nnn;
    }

    fn jpv0_bnnn(&mut self, nnn: u16) {
        self.pc = (self.reg(0x00) as u16) + nnn;
    }

    fn rnd_cxkk(&mut self, x: u8, kk: u8) {}

    fn drw_dxyn(&mut self, x: u8, y: u8, n: u8) {}

    fn skp_ex9e(&mut self, x: u8) {}
    fn sknp_exa1(&mut self, x: u8) {}
    fn ld_fx07(&mut self, x: u8) {
        self.v_reg[x as usize] = self.delay_timer;
    }
    fn ld_fx0a(&mut self, x: u8) {}
    fn ld_fx15(&mut self, x: u8) {
        self.delay_timer = self.reg(x);
    }
    fn ld_fx18(&mut self, x: u8) {
        self.sound_timer = self.reg(x);
    }
    fn add_fx1e(&mut self, x: u8) {
        self.i_reg += self.reg(x) as u16;
    }
    fn ld_fx29(&mut self, x: u8) {}
    fn ld_fx33(&mut self, x: u8) {
        let dec = self.reg(x);

        let ones = dec % 10;
        let tens = (dec / 10) % 10;
        let hundres = (dec / 100) % 10;

        todo!("spec unclear")
    }
    fn ld_fx55(&mut self, x: u8) {}
    fn ld_fx65(&mut self, x: u8) {}

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

    fn xkk(opcode: u16) -> (u8, u8) {
        let x = ((opcode & 0x0F00) >> 8) as u8;
        let kk = (opcode & 0x00FF) as u8;
        (x, kk)
    }

    fn kk(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }

    fn xyz(opcode: u16) -> (usize, usize, u16) {
        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;
        let z = opcode & 0x000F;
        (x.into(), y.into(), z)
    }

    fn reg(&self, x: u8) -> u8 {
        self.v_reg[x as usize]
    }
}
