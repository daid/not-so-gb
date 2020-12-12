use std::fs::File;
use std::io::Read;

#[path = "video.rs"]
mod video;

pub struct CPU {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    pc: u16,
    sp: u16,

    zero: bool,
    carry: bool,

    ime: bool,

    rom: Vec<u8>,

    wram: [u8; 0x2000],
    hram: [u8; 0x80],

    reg_if: u8,
    reg_ie: u8,

    rombank_offset: usize,

    video: video::Video,
    start_time: std::time::Instant,
}

impl CPU {
    pub fn new(filename: &str) -> CPU {
        let mut file = File::open(filename).unwrap();

        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        return CPU {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,

            pc: 0x0150,
            sp: 0xffc0,

            zero: true,
            carry: true,

            ime: true,

            rom: data,

            wram: [0; 0x2000],
            hram: [0; 0x80],

            reg_if: 0,
            reg_ie: 0,

            rombank_offset: 0x4000,

            video: video::Video::new(),
            start_time: std::time::Instant::now(),
        };
    }

    pub fn step(&mut self) {
        let pc = self.pc_inc();
        match self.read_mem(pc) {
            0x00 => {}
            0x01 => {
                self.set_bc(self.read_mem16(self.pc));
                self.pc += 2;
            }
            0x04 => {
                self.b = self.b.wrapping_add(1);
                self.zero = self.b == 0;
            }
            0x05 => {
                self.b = self.b.wrapping_sub(1);
                self.zero = self.b == 0;
            }
            0x06 => {
                self.b = self.read_mem(self.pc);
                self.pc += 1;
            }
            0x07 => {
                self.carry = (self.a & 0x80) == 0x80;
                self.a = (self.a >> 7) | ((self.a & 0x7f) << 1);
            }
            0x09 => self.set_hl(self.hl().wrapping_add(self.bc())), //carry flag...
            0x0b => self.set_bc(self.bc().wrapping_sub(1)),
            0x0c => {
                self.c = self.c.wrapping_add(1);
                self.zero = self.c == 0;
            }
            0x0d => {
                self.d = self.d.wrapping_sub(1);
                self.zero = self.d == 0;
            }
            0x0e => {
                self.c = self.read_mem(self.pc);
                self.pc += 1;
            }
            0x11 => {
                self.set_de(self.read_mem16(self.pc));
                self.pc += 2;
            }
            0x13 => self.set_bc(self.bc().wrapping_add(1)),
            0x15 => {
                self.d = self.d.wrapping_sub(1);
                self.zero = self.d == 0;
            }
            0x16 => {
                self.d = self.read_mem(self.pc);
                self.pc += 1;
            }
            0x18 => self.instr_jr(),
            0x19 => self.set_hl(self.hl().wrapping_add(self.de())), //carry flag...
            0x1a => self.a = self.read_mem(self.de()),
            0x1b => self.set_bc(self.bc().wrapping_sub(1)),
            0x1d => {
                self.e = self.e.wrapping_sub(1);
                self.zero = self.e == 0;
            }
            0x1e => {
                self.e = self.read_mem(self.pc);
                self.pc += 1;
            }
            0x20 => {
                if !self.zero {
                    self.instr_jr();
                } else {
                    self.pc += 1;
                }
            }
            0x21 => {
                self.set_hl(self.read_mem16(self.pc));
                self.pc += 2;
            }
            0x22 => {
                self.write_mem(self.hl(), self.a);
                self.set_hl(self.hl().wrapping_add(1))
            }
            0x23 => self.set_hl(self.hl().wrapping_add(1)),
            0x26 => {
                self.h = self.read_mem(self.pc);
                self.pc += 1;
            }
            0x28 => {
                if self.zero {
                    self.instr_jr();
                } else {
                    self.pc_inc();
                }
            }
            0x29 => self.set_hl(self.hl().wrapping_add(self.hl())), //carry flag...
            0x2a => {
                self.a = self.read_mem(self.hl());
                self.set_hl(self.hl().wrapping_add(1))
            }
            0x2b => self.set_hl(self.hl().wrapping_sub(1)),
            0x2f => self.a = !self.a,
            0x30 => {
                if !self.carry {
                    self.instr_jr();
                } else {
                    self.pc += 1;
                }
            }
            0x31 => {
                self.sp = self.read_mem16(self.pc);
                self.pc += 2;
            }
            0x34 => {
                let v = self.read_mem(self.hl()).wrapping_add(1);
                self.zero = v == 0;
                self.write_mem(self.hl(), 1);
            }
            0x35 => {
                let v = self.read_mem(self.hl()).wrapping_sub(1);
                self.zero = v == 0;
                self.write_mem(self.hl(), 1);
            }
            0x36 => {
                self.write_mem(self.hl(), self.read_mem(self.pc));
                self.pc += 1;
            }
            0x37 => self.carry = true,
            0x38 => {
                if self.carry {
                    self.instr_jr();
                } else {
                    self.pc_inc();
                }
            }
            0x3a => {
                self.a = self.read_mem(self.hl());
                self.set_hl(self.hl().wrapping_sub(1))
            }
            0x3c => {
                self.a = self.a.wrapping_add(1);
                self.zero = self.a == 0;
            }
            0x3d => {
                self.a = self.a.wrapping_sub(1);
                self.zero = self.a == 0;
            }
            0x3e => {
                self.a = self.read_mem(self.pc);
                self.pc += 1;
            }
            0x3f => self.carry = !self.carry,
            0x41 => self.b = self.c,
            0x42 => self.b = self.d,
            0x46 => self.b = self.read_mem(self.hl()),
            0x47 => self.b = self.a,
            0x4a => self.c = self.d,
            0x4f => self.c = self.a,
            0x50 => self.d = self.b,
            0x51 => self.d = self.c,
            0x54 => self.d = self.h,
            0x56 => self.d = self.read_mem(self.hl()),
            0x57 => self.d = self.a,
            0x5d => self.e = self.l,
            0x5e => self.e = self.read_mem(self.hl()),
            0x5f => self.e = self.a,
            0x62 => self.h = self.d,
            0x67 => self.h = self.a,
            0x68 => self.l = self.b,
            0x6b => self.l = self.e,
            0x6e => self.l = self.read_mem(self.hl()),
            0x6f => self.l = self.a,
            0x70 => self.write_mem(self.hl(), self.b),
            0x71 => self.write_mem(self.hl(), self.c),
            0x72 => self.write_mem(self.hl(), self.d),
            0x73 => self.write_mem(self.hl(), self.e),
            0x74 => self.write_mem(self.hl(), self.h),
            0x75 => self.write_mem(self.hl(), self.l),
            0x76 => {}
            0x77 => self.write_mem(self.hl(), self.a),
            0x78 => self.a = self.b,
            0x79 => self.a = self.c,
            0x7a => self.a = self.d,
            0x7b => self.a = self.e,
            0x7c => self.a = self.h,
            0x7d => self.a = self.l,
            0x7e => self.a = self.read_mem(self.hl()),
            0x80 => self.instr_add(self.b),
            0x81 => self.instr_add(self.c),
            0x82 => self.instr_add(self.d),
            0x83 => self.instr_add(self.e),
            0x85 => self.instr_add(self.l),
            0x87 => self.instr_add(self.a),
            0x88 => self.instr_adc(self.b),
            0x98 => self.instr_sbc(self.b),
            0xa0 => self.instr_and(self.b),
            0xa3 => self.instr_and(self.e),
            0xa6 => self.instr_and(self.read_mem(self.hl())),
            0xa7 => self.instr_and(self.a),
            0xa8 => self.instr_xor(self.b),
            0xaf => {
                self.a = 0;
                self.zero = true;
                self.carry = false;
            }
            0xb0 => self.instr_or(self.b),
            0xb1 => self.instr_or(self.c),
            0xb2 => self.instr_or(self.d),
            0xb6 => self.instr_or(self.read_mem(self.hl())),
            0xb8 => self.instr_cp(self.b),
            0xb9 => self.instr_cp(self.c),
            0xbe => self.instr_cp(self.read_mem(self.hl())),
            0xc0 => {
                if !self.zero {
                    self.instr_ret()
                }
            }
            0xc1 => {
                let value = self.pop();
                self.set_bc(value);
            }
            0xc2 => {
                if !self.zero {
                    self.pc = self.read_mem16(self.pc)
                } else {
                    self.pc += 2;
                }
            }
            0xc3 => self.pc = self.read_mem16(self.pc),
            0xc5 => self.push(self.bc()),
            0xc8 => {
                if self.zero {
                    self.instr_ret();
                }
            }
            0xc9 => self.instr_ret(),
            0xca => {
                if self.zero {
                    self.pc = self.read_mem16(self.pc)
                } else {
                    self.pc += 2;
                }
            }
            0xcc => {
                if self.zero {
                    self.instr_call();
                }
            }
            0xcd => self.instr_call(),
            0xd1 => {
                let value = self.pop();
                self.set_de(value)
            }
            0xd2 => {
                if !self.carry {
                    self.pc = self.read_mem16(self.pc)
                } else {
                    self.pc += 2;
                }
            }
            0xd5 => self.push(self.de()),
            0xd9 => {
                self.ime = true;
                self.instr_ret();
            }
            0xda => {
                if self.carry {
                    self.pc = self.read_mem16(self.pc)
                } else {
                    self.pc += 2;
                }
            }
            0xe0 => {
                let addr = self.read_mem(self.pc);
                self.pc += 1;
                self.write_mem(addr as u16 | 0xff00, self.a);
            }
            0xe1 => {
                let value = self.pop();
                self.set_hl(value);
            }
            0xe2 => self.write_mem(self.c as u16 | 0xff00, self.a),
            0xe5 => self.push(self.hl()),
            0xe6 => {
                self.instr_and(self.read_mem(self.pc));
                self.pc += 1;
            }
            0xe9 => self.pc = self.hl(),
            0xea => {
                let addr = self.read_mem16(self.pc);
                self.pc += 2;
                self.write_mem(addr, self.a);
            }
            0xf0 => {
                let addr = self.read_mem(self.pc);
                self.pc += 1;
                self.a = self.read_mem(addr as u16 | 0xff00);
            }
            0xf1 => {
                let value = self.pop();
                self.set_af(value);
            }
            0xf3 => self.ime = false,
            0xf5 => self.push(self.af()),
            0xf6 => {
                self.instr_or(self.read_mem(self.pc));
                self.pc += 1;
            }
            0xfa => {
                let addr = self.read_mem16(self.pc);
                self.pc += 2;
                self.a = self.read_mem(addr);
            }
            0xfb => self.ime = true,
            0xfe => {
                let addr = self.pc_inc();
                self.instr_cp(self.read_mem(addr));
            }
            0xcb => {
                let next_pc = self.pc_inc();
                match self.read_mem(next_pc) {
                    0x12 => self.d = self.instr_rl(self.d),
                    0x1b => self.e = self.instr_rr(self.e),
                    0x23 => self.e = self.instr_sla(self.e),
                    0x2a => self.e = self.instr_sra(self.e),
                    0x30 => self.b = self.instr_swap(self.b),
                    0x31 => self.c = self.instr_swap(self.c),
                    0x32 => self.d = self.instr_swap(self.d),
                    0x33 => self.e = self.instr_swap(self.e),
                    0x34 => self.h = self.instr_swap(self.h),
                    0x35 => self.l = self.instr_swap(self.l),
                    0x36 => {
                        let v = self.instr_swap(self.read_mem(self.hl()));
                        self.write_mem(self.hl(), v);
                    }
                    0x37 => self.a = self.instr_swap(self.a),
                    0x3f => self.a = self.instr_srl(self.a),
                    0x46 => self.zero = (self.read_mem(self.hl()) & (1 << 0)) == 0,
                    0x47 => self.zero = (self.a & (1 << 0)) == 0,
                    0x4e => self.zero = (self.read_mem(self.hl()) & (1 << 1)) == 0,
                    0x4f => self.zero = (self.a & (1 << 1)) == 0,
                    0x50 => self.zero = (self.b & (1 << 2)) == 0,
                    0x51 => self.zero = (self.c & (1 << 2)) == 0,
                    0x52 => self.zero = (self.d & (1 << 2)) == 0,
                    0x53 => self.zero = (self.e & (1 << 2)) == 0,
                    0x54 => self.zero = (self.h & (1 << 2)) == 0,
                    0x55 => self.zero = (self.l & (1 << 2)) == 0,
                    0x56 => self.zero = (self.read_mem(self.hl()) & (1 << 2)) == 0,
                    0x57 => self.zero = (self.a & (1 << 2)) == 0,
                    0x58 => self.zero = (self.b & (1 << 3)) == 0,
                    0x59 => self.zero = (self.c & (1 << 3)) == 0,
                    0x5a => self.zero = (self.d & (1 << 3)) == 0,
                    0x5b => self.zero = (self.e & (1 << 3)) == 0,
                    0x5c => self.zero = (self.h & (1 << 3)) == 0,
                    0x5d => self.zero = (self.l & (1 << 3)) == 0,
                    0x5e => self.zero = (self.read_mem(self.hl()) & (1 << 3)) == 0,
                    0x5f => self.zero = (self.a & (1 << 3)) == 0,
                    0x66 => self.zero = (self.read_mem(self.hl()) & (1 << 4)) == 0,
                    0x6f => self.zero = (self.a & (1 << 5)) == 0,
                    0x70 => self.zero = (self.b & (1 << 6)) == 0,
                    0x71 => self.zero = (self.c & (1 << 6)) == 0,
                    0x72 => self.zero = (self.d & (1 << 6)) == 0,
                    0x73 => self.zero = (self.e & (1 << 6)) == 0,
                    0x74 => self.zero = (self.h & (1 << 6)) == 0,
                    0x75 => self.zero = (self.l & (1 << 6)) == 0,
                    0x76 => self.zero = (self.read_mem(self.hl()) & (1 << 6)) == 0,
                    0x77 => self.zero = (self.a & (1 << 6)) == 0,
                    0x78 => self.zero = (self.b & (1 << 7)) == 0,
                    0x79 => self.zero = (self.c & (1 << 7)) == 0,
                    0x7a => self.zero = (self.d & (1 << 7)) == 0,
                    0x7b => self.zero = (self.e & (1 << 7)) == 0,
                    0x7c => self.zero = (self.h & (1 << 7)) == 0,
                    0x7d => self.zero = (self.l & (1 << 7)) == 0,
                    0x7e => self.zero = (self.read_mem(self.hl()) & (1 << 7)) == 0,
                    0x7f => self.zero = (self.a & (1 << 7)) == 0,
                    0x80 => self.b &= !(1 << 0),
                    0x81 => self.c &= !(1 << 0),
                    0x82 => self.d &= !(1 << 0),
                    0x83 => self.e &= !(1 << 0),
                    0x84 => self.h &= !(1 << 0),
                    0x85 => self.l &= !(1 << 0),
                    0x86 => self.write_mem(self.hl(), self.read_mem(self.hl()) & !(1 << 0)),
                    0x87 => self.a &= !(1 << 0),
                    0x88 => self.b &= !(1 << 1),
                    0x89 => self.c &= !(1 << 1),
                    0x8a => self.d &= !(1 << 1),
                    0x8b => self.e &= !(1 << 1),
                    0x8c => self.h &= !(1 << 1),
                    0x8d => self.l &= !(1 << 1),
                    0x8e => self.write_mem(self.hl(), self.read_mem(self.hl()) & !(1 << 1)),
                    0x8f => self.a &= !(1 << 1),
                    0x90 => self.b &= !(1 << 2),
                    0x91 => self.c &= !(1 << 2),
                    0x92 => self.d &= !(1 << 2),
                    0x93 => self.e &= !(1 << 2),
                    0x94 => self.h &= !(1 << 2),
                    0x95 => self.l &= !(1 << 2),
                    0x96 => self.write_mem(self.hl(), self.read_mem(self.hl()) & !(1 << 2)),
                    0x97 => self.a &= !(1 << 2),
                    0xa2 => self.d &= !(1 << 4),
                    0xa6 => self.write_mem(self.hl(), self.read_mem(self.hl()) & !(1 << 4)),
                    0xae => self.write_mem(self.hl(), self.read_mem(self.hl()) & !(1 << 5)),
                    0xaf => self.a &= !(1 << 5),
                    0xd0 => self.b |= 1 << 2,
                    0xd1 => self.c |= 1 << 2,
                    0xd2 => self.d |= 1 << 2,
                    0xd3 => self.e |= 1 << 2,
                    0xd4 => self.h |= 1 << 2,
                    0xd5 => self.l |= 1 << 2,
                    0xd6 => self.write_mem(self.hl(), self.read_mem(self.hl()) | (1 << 2)),
                    0xd7 => self.a |= 1 << 2,
                    0xd8 => self.b |= 1 << 3,
                    0xd9 => self.c |= 1 << 3,
                    0xda => self.d |= 1 << 3,
                    0xdb => self.e |= 1 << 3,
                    0xdc => self.h |= 1 << 3,
                    0xdd => self.l |= 1 << 3,
                    0xde => self.write_mem(self.hl(), self.read_mem(self.hl()) | (1 << 3)),
                    0xdf => self.a |= 1 << 3,
                    _ => panic!(
                        "Unknown opcode: {:04x}:cb:{:02x}",
                        pc,
                        self.read_mem(next_pc)
                    ),
                }
            }
            _ => panic!("Unknown opcode: {:04x}:{:02x}", pc, self.read_mem(pc)),
        }

        self.video.step();
        if self.video.vblank_interrupt {
            self.video.vblank_interrupt = false;
            self.reg_if |= 0x01;
        }

        if self.ime && (self.reg_if & self.reg_ie) != 0x00 {
            if (self.reg_if & self.reg_ie & 0x01) == 0x01 {
                self.reg_if &= !0x01;
                self.ime = false;
                self.push(self.pc);
                self.pc = 0x0040;
            }
        }
    }

    fn read_mem(&self, addr: u16) -> u8 {
        if addr < 0x4000 {
            return self.rom.get(addr as usize).copied().unwrap_or(0);
        } else if addr < 0x8000 {
            return self
                .rom
                .get(addr as usize - 0x4000 + self.rombank_offset)
                .copied()
                .unwrap_or(0);
        } else if addr < 0xa000 {
            return self.video.vram[addr as usize - 0x8000];
        } else if addr >= 0xc000 && addr < 0xe000 {
            return self.wram[addr as usize - 0xc000];
        } else if addr == 0xff00 {
            return 0xff;
        } else if addr == 0xff04 {
            return (self.start_time.elapsed().as_nanos() & 0xff) as u8;
        } else if addr == 0xff0f {
            return self.reg_if;
        } else if addr == 0xff25 {
            return 0;
        } else if addr == 0xff40 {
            return self.video.lcdc;
        } else if addr == 0xff41 {
            return self.video.stat;
        } else if addr == 0xff44 {
            return self.video.ly;
        } else if addr >= 0xff80 && addr < 0xffff {
            return self.hram[addr as usize - 0xff80];
        } else if addr == 0xffff {
            return self.reg_ie;
        }
        panic!("Read unknown addr: {:04x}", addr);
        return 0;
    }

    fn write_mem(&mut self, addr: u16, value: u8) {
        if addr == 0x2000 {
            self.rombank_offset = value as usize * 0x4000;
        } else if addr >= 0xc000 && addr < 0xe000 {
            self.wram[addr as usize - 0xc000] = value;
        } else if addr >= 0x8000 && addr < 0xa000 {
            self.video.vram[addr as usize - 0x8000] = value;
        } else if addr >= 0xff80 && addr < 0xffff {
            self.hram[addr as usize - 0xff80] = value;
        } else if addr == 0xff00 {
        } else if addr == 0xff01 {
        } else if addr == 0xff02 {
        } else if addr == 0xff06 {
        } else if addr == 0xff07 {
        } else if addr == 0xff0f {
            self.reg_if = value;
        } else if addr == 0xff10 {
        } else if addr == 0xff12 {
        } else if addr == 0xff14 {
        } else if addr == 0xff17 {
        } else if addr == 0xff19 {
        } else if addr == 0xff1a {
        } else if addr == 0xff1c {
        } else if addr == 0xff21 {
        } else if addr == 0xff23 {
        } else if addr == 0xff24 {
        } else if addr == 0xff25 {
        } else if addr == 0xff26 {
        } else if addr == 0xff40 {
            self.video.lcdc = value;
        } else if addr == 0xff41 {
            self.video.stat = value;
        } else if addr == 0xff42 {
        } else if addr == 0xff43 {
        } else if addr == 0xff46 {
            for n in 0..40 * 4 {
                self.video.oam[n] = self.read_mem((value as u16) << 8 | n as u16);
            }
        } else if addr == 0xff47 {
        } else if addr == 0xff48 {
        } else if addr == 0xff49 {
        } else if addr == 0xff4a {
        } else if addr == 0xff4b {
        } else if addr == 0xffff {
            self.reg_ie = value;
        } else {
            panic!("Write unknown addr: {:04x}", addr);
        }
    }

    fn read_mem16(&self, addr: u16) -> u16 {
        let low = self.read_mem(addr);
        let high = self.read_mem(addr + 1);
        return low as u16 | ((high as u16) << 8);
    }

    fn write_mem16(&mut self, addr: u16, value: u16) {
        self.write_mem(addr, (value & 0xff) as u8);
        self.write_mem(addr + 1, (value >> 8) as u8);
    }

    fn pc_inc(&mut self) -> u16 {
        let result = self.pc;
        self.pc += 1;
        return result;
    }

    fn set_af(&mut self, value: u16) {
        self.a = (value & 0xff) as u8;
        self.zero = (value & 0x100) == 0x100;
        self.carry = (value & 0x200) == 0x200;
    }

    fn af(&self) -> u16 {
        let mut result = self.a as u16;
        if self.zero {
            result |= 0x100;
        }
        if self.carry {
            result |= 0x200;
        }
        return result;
    }

    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xff) as u8;
    }

    fn bc(&self) -> u16 {
        return ((self.b as u16) << 8) | self.c as u16;
    }

    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xff) as u8;
    }

    fn de(&self) -> u16 {
        return ((self.d as u16) << 8) | self.e as u16;
    }

    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xff) as u8;
    }

    fn hl(&self) -> u16 {
        return ((self.h as u16) << 8) | self.l as u16;
    }

    fn push(&mut self, value: u16) {
        self.sp -= 2;
        self.write_mem16(self.sp, value);
    }

    fn pop(&mut self) -> u16 {
        let result = self.read_mem16(self.sp);
        self.sp += 2;
        return result;
    }

    fn instr_jr(&mut self) {
        let addr = self.pc_inc();
        let mut offset = self.read_mem(addr) as u16;
        if (offset & 0x80) == 0x80 {
            offset |= 0xff00;
        }
        self.pc = self.pc.wrapping_add(offset);
    }

    fn instr_add(&mut self, value: u8) {
        let result = self.a as u16 + value as u16;
        self.a = (result & 0xff) as u8;
        self.zero = self.a == 0;
        self.carry = result > 0xff;
    }

    fn instr_adc(&mut self, value: u8) {
        let mut result = self.a as u16 + value as u16;
        if self.carry {
            result += 1;
        }
        self.a = (result & 0xff) as u8;
        self.zero = self.a == 0;
        self.carry = result > 0xff;
    }

    fn instr_sub(&mut self, value: u8) {
        let result = (self.a as u16).wrapping_sub(value as u16);
        self.a = (result & 0xff) as u8;
        self.zero = self.a == 0;
        self.carry = result > 0xff;
    }

    fn instr_sbc(&mut self, value: u8) {
        let mut result = (self.a as u16).wrapping_sub(value as u16);
        if self.carry {
            result = result.wrapping_sub(1);
        }
        self.a = (result & 0xff) as u8;
        self.zero = self.a == 0;
        self.carry = result > 0xff;
    }

    fn instr_and(&mut self, value: u8) {
        self.a &= value;
        self.zero = self.a == 0;
        self.carry = false;
    }

    fn instr_or(&mut self, value: u8) {
        self.a |= value;
        self.zero = self.a == 0;
        self.carry = false;
    }

    fn instr_xor(&mut self, value: u8) {
        self.a ^= value;
        self.zero = self.a == 0;
        self.carry = false;
    }

    fn instr_cp(&mut self, value: u8) {
        self.zero = value == self.a;
        self.carry = value > self.a;
    }

    fn instr_call(&mut self) {
        self.push(self.pc + 2);
        self.pc = self.read_mem16(self.pc);
    }

    fn instr_ret(&mut self) {
        self.pc = self.pop();
    }

    fn instr_sla(&mut self, value: u8) -> u8 {
        self.carry = (value & 0x80) == 0x80;
        self.zero = (value & 0x7f) << 1 == 0;
        return (value & 0x7f) << 1;
    }

    fn instr_sra(&mut self, value: u8) -> u8 {
        let result = (value >> 1) | (value & 0x80);
        self.carry = (value & 0x01) == 0x01;
        self.zero = result == 0;
        return result;
    }

    fn instr_srl(&mut self, value: u8) -> u8 {
        let result = value >> 1;
        self.carry = (value & 0x01) == 0x01;
        self.zero = result == 0;
        return result;
    }

    fn instr_rl(&mut self, value: u8) -> u8 {
        let mut result = (value & 0x7f) << 1;
        if self.carry {
            result |= 1;
        }
        self.carry = (value & 0x80) == 0x80;
        self.zero = result == 0;
        return result;
    }

    fn instr_rr(&mut self, value: u8) -> u8 {
        let mut result = value >> 1;
        if self.carry {
            result |= 0x80;
        }
        self.carry = (value & 0x01) == 0x01;
        self.zero = result == 0;
        return result;
    }

    fn instr_swap(&mut self, value: u8) -> u8 {
        self.zero = value == 0;
        return value >> 4 | (value & 0x0f) << 4;
    }
}
