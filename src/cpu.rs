use std::fs::File;
use std::io::Read;

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

    rombank_offset: usize,
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

            rombank_offset: 0x4000,
        };
    }

    pub fn step(&mut self) {
        let pc = self.pc_inc();
        match self.read_mem(pc) {
            0x00 => {}
            0x18 => self.instr_jr(),
            0x28 => {
                if self.zero {
                    self.instr_jr();
                } else {
                    self.pc_inc();
                }
            }
            0x3e => {
                self.a = self.read_mem(self.pc);
                self.pc += 1;
            }
            0x46 => self.b = self.read_mem(self.hl()),
            0xaf => {
                self.a = 0;
                self.zero = true;
                self.carry = false;
            }
            0xc3 => self.pc = self.read_mem16(self.pc),
            0xcd => self.instr_call(),
            0xe0 => {
                let addr = self.read_mem(self.pc);
                self.pc += 1;
                self.write_mem(addr as u16 | 0xff00, self.a);
            }
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
            0xf3 => self.ime = false,
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

            _ => panic!("Unknown opcode: {:04x}:{:02x}", pc, self.read_mem(pc)),
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
        }
        panic!("Read unknown addr: {:04x}", addr);
        return 0;
    }

    fn write_mem(&mut self, addr: u16, value: u8) {
        if addr >= 0xc000 && addr < 0xe000 {
            self.wram[addr as usize - 0xc000] = value;
        } else if addr >= 0xff80 && addr < 0xffff {
            self.hram[addr as usize - 0xff80] = value;
        } else if addr == 0xff01 {
        } else if addr == 0xff02 {
        } else if addr == 0xff06 {
        } else if addr == 0xff07 {
        } else if addr == 0xff0f {
        } else if addr == 0xff40 {
        } else if addr == 0xff42 {
        } else if addr == 0xff43 {
        } else if addr == 0xff47 {
        } else if addr == 0xff48 {
        } else if addr == 0xff49 {
        } else if addr == 0xff4a {
        } else if addr == 0xff4b {
        } else if addr == 0xffff {
        } else {
            panic!("Write unknown addr: {:04x}", addr);
        }
    }

    fn read_mem16(&mut self, addr: u16) -> u16 {
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

    fn hl(&self) -> u16 {
        return ((self.h as u16) << 8) | self.l as u16;
    }

    fn instr_jr(&mut self) {
        let addr = self.pc_inc();
        let mut offset = self.read_mem(addr) as u16;
        if (offset & 0x80) == 0x80 {
            offset |= 0xff00;
        }
        self.pc = self.pc.wrapping_add(offset);
    }

    fn instr_cp(&mut self, value: u8) {
        self.zero = value == self.a;
        self.carry = value > self.a;
    }

    fn instr_call(&mut self) {
        self.sp -= 2;
        self.write_mem16(self.sp, self.pc + 2);
        self.pc = self.read_mem16(self.pc);
    }
}
