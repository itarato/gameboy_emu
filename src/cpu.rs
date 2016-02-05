use bus::{Bus};

macro_rules! rl {
    ($_self:expr, $reg:ident) => (
        {
            let new_carry = $_self.$reg >> 7;
            $_self.$reg = ($_self.$reg << 1) | $_self.flag.c_carry as u8;
            $_self.$reg = 1;
            $_self.flag.c_carry = new_carry == 1;
            $_self.flag.z_zero = $_self.$reg == 0;
            $_self.flag.n_substract = false;
            $_self.flag.h_half_carry = false;
        }
    )
}

macro_rules! dec_n {
    ($_self:expr, $reg:ident) => (
        {
            $_self.$reg = if $_self.$reg == 0 { 0xFF } else { $_self.$reg - 1 };
            // TODO verify if this is a conditional set or always. Now it's always.
            $_self.flag.z_zero = $_self.$reg == 0;
            $_self.flag.n_substract = true;
            // TODO verify if this a definite set or only when bit 3 == 1
            $_self.flag.h_half_carry = $_self.$reg >> 3 & 1 == 1;
        }
    )
}

macro_rules! inc_n {
    ($_self:expr, $reg:ident) => (
        {
            $_self.$reg = if $_self.$reg == 0xFF { 0 } else { $_self.$reg + 1 };
            // TODO verify if this is a conditional set or always. Now it's always.
            $_self.flag.z_zero = $_self.$reg == 0;
            $_self.flag.n_substract = false;
            // TODO verify if this a definite set or only when bit 3 == 1
            $_self.flag.h_half_carry = $_self.$reg >> 3 & 1 == 1;
        }
    )
}

macro_rules! inc_dd {
    ($reg_hi:expr, $reg_lo:expr) => (
        {
            let (hi, lo) = inc_dd($reg_hi, $reg_lo);
            $reg_hi = hi;
            $reg_lo = lo;
        }
    )
}

macro_rules! dec_dd {
    ($reg_hi:expr, $reg_lo:expr) => (
        {
            let (hi, lo) = dec_dd($reg_hi, $reg_lo);
            $reg_hi = hi;
            $reg_lo = lo;
        }
    )
}

const STACK_TOP: u16 = 0xFFFE;
// TOOD verify it's true
const STACK_BOTTOM: u16 = 0xFF80;

fn u16_to_hi_lo(dd: u16) -> (u8, u8) {
    ((dd >> 8) as u8, (dd & 0xFF) as u8)
}

fn hi_lo_to_u16(hi:u8, lo:u8) -> u16 {
    (hi as u16) << 8 | lo as u16
}

fn inc_dd(hi: u8, lo: u8) -> (u8, u8) {
    let mut val: u16 = hi_lo_to_u16(hi, lo);
    val += 1;
    u16_to_hi_lo(val)
}

fn dec_dd(hi: u8, lo: u8) -> (u8, u8) {
    let mut val: u16 = hi_lo_to_u16(hi, lo);
    val -= 1;
    u16_to_hi_lo(val)
}

#[derive(Default, Debug)]
struct Flags {
    z_zero: bool,
    n_substract: bool,
    h_half_carry: bool,
    c_carry: bool,
}

#[derive(Default, Debug)]
pub struct CPU {
    // Main register set.
    acc: u8,
    flag: Flags,

    // General purpose registers.
    b: u8,
    d: u8,
    h: u8,

    c: u8,
    e: u8,
    l: u8,

    // Special purpose registers.
    sp: u16,
    pc: u16,
}

impl CPU {
    pub fn new() -> CPU {
        CPU::default()
    }

    pub fn reset(&mut self) {
        // Point to first instruction.
        self.pc = 0x0000;
    }

    pub fn next_instruction(&mut self, bus: &mut Bus)  {
        let opcode = self.read_opcode(bus);
        println!("Read opcode {:#x} ({:#b}) at PC {:#x} ({})", opcode, opcode, self.pc - 1, self.pc - 1);

        match opcode {
            // CALL n.
            0xCD => {
                let (addr_hi, addr_lo) = u16_to_hi_lo(self.pc);
                // TODO review, http://www.devrs.com/gb/files/instr.txt does not mention SP adjustment here
                self.stack_push(addr_hi, bus);
                self.stack_push(addr_lo, bus);

                let (vlow, vhigh) = self.read_low_high(bus);
                self.pc = hi_lo_to_u16(vhigh, vlow);
            },

            // DEC B.
            0x05 => dec_n!(self, b),
            // DEC D.
            0x15 => dec_n!(self, d),
            // DEC H.
            0x25 => dec_n!(self, h),
            // DEC C.
            0x0D => dec_n!(self, c),
            // DEC E.
            0x1D => dec_n!(self, e),
            // DEC L.
            0x2D => dec_n!(self, l),
            // DEC A.
            0x3D => dec_n!(self, acc),
            // DEC BC.
            0x0B => dec_dd!(self.b, self.c),
            // DEC DE.
            0x1B => dec_dd!(self.d, self.e),
            // DEC HL.
            0x2B => dec_dd!(self.h, self.l),

            // INC B.
            0x04 => inc_n!(self, b),
            // INC D.
            0x14 => inc_n!(self, d),
            // INC H.
            0x24 => inc_n!(self, h),
            // INC C.
            0x0C => inc_n!(self, c),
            // INC E.
            0x1C => inc_n!(self, e),
            // INC L.
            0x2C => inc_n!(self, l),
            // INC A.
            0x3C => inc_n!(self, acc),
            // INC BC.
            0x03 => inc_dd!(self.b, self.c),
            // INC DE.
            0x13 => inc_dd!(self.d, self.e),
            // INC HL.
            0x23 => inc_dd!(self.h, self.l),

            // JR NZ,r8.
            0x20 => {
                let addr = self.read_byte(bus);
                if !self.flag.z_zero {
                    self.pc = ((self.pc as i16) + ((addr as i8) as i16)) as u16;
                }
            },

            // LD BC,d16.
            0x01 => {
                let (vlow, vhigh) = self.read_low_high(bus);
                self.c = vlow;
                self.b = vhigh;
            },
            // LD C,d8.
            0x0E => self.c = self.read_byte(bus),
            // LD DE,d16.
            0x11 => {
                let (vlow, vhigh) = self.read_low_high(bus);
                self.e = vlow;
                self.d = vhigh;
            },
            // LD A,(DE).
            0x1A => {
                let addr = (self.d as u16) << 8 | (self.e as u16);
                self.acc = bus.read_byte(addr as usize);
            }
            // LD HL,d16.
            0x21 => {
                let (vlow, vhigh) = self.read_low_high(bus);
                self.l = vlow;
                self.h = vhigh;
            },
            // LD SP,d16.
            0x31 => {
                let (vlow, vhigh) = self.read_low_high(bus);
                self.sp = (vhigh as u16) << 8 | (vlow as u16);
            },
            // LD (HL-),A.
            0x32 => {
                let mut addr = hi_lo_to_u16(self.h, self.l);
                bus.write_byte(addr as usize, self.acc);

                assert!(addr > 0, "Address reg HL is zero, cannot be decremented");
                addr -= 1;

                // TODO make it a func or macro.
                self.h = (addr >> 8) as u8;
                self.l = (addr & 0xFF) as u8;
            },
            // LD (HL+),A.
            0x22 => {
                let mut addr = hi_lo_to_u16(self.h, self.l);
                bus.write_byte(addr as usize, self.acc);

                assert!(addr < 0xFFFF, "Address reg HL is max (0xFFFF), cannot be incremented");
                addr += 1;

                // TODO make it a func or macro.
                self.h = (addr >> 8) as u8;
                self.l = (addr & 0xFF) as u8;
            },
            // LD A,d8.
            0x3E => self.acc = self.read_byte(bus),
            // LD C,A.
            0x4F => self.c = self.acc,
            // LD (HL),A.
            0x77 => {
                let addr = ((self.h as u16) << 8) | (self.l as u16);
                bus.write_byte(addr as usize, self.acc);
            },
            // LDH (n),A.
            0xE0 => {
                let addr = self.read_byte(bus);
                bus.write_byte((0xFF00 + (addr as u16)) as usize, self.acc);
            },
            // LD (C),A.
            0xE2 => {
                // let offs = self.read_byte(bus);
                // bus.write_byte((offs + self.c) as usize, self.acc);
                // There is some contradiciton here.
                // http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html mentions LD (C),A is a 2 byte op.
                // However it doesn't refer to a loaded byte as well as http://gbdev.gg8.se/wiki/articles/Gameboy_Bootstrap_ROM
                // says its a 1 byte op with fixed signing. We follow the latter now.
                bus.write_byte((0xFF00 + (self.c as u16)) as usize, self.acc);
            },
            // LD B,d8.
            0x06 => self.b = self.read_byte(bus),

            // POP BC.
            0xC1 => {
                self.c = self.stack_pop(bus);
                self.b = self.stack_pop(bus);
            },

            // Prefix CB.
            0xCB => self.exec_prefixed_instruction(opcode, bus),

            // PUSH BC.
            0xC5 => {
                let (b, c) = (self.b, self.c);
                self.stack_push(b, bus);
                self.stack_push(c, bus);
            },

            // RLA.
            0x17 => rl!(self, acc),

            // XOR B.
            0xA8 => self.b ^= self.b,
            // XOR C.
            0xA9 => self.c ^= self.c,
            // XOR D.
            0xAA => self.d ^= self.d,
            // XOR E.
            0xAB => self.e ^= self.e,
            // XOR H.
            0xAC => self.h ^= self.h,
            // XOR L.
            0xAD => self.l ^= self.l,
            // XOR A.
            0xAF => self.acc ^= self.acc,

            _ => panic!("Unknown opcode {:#x} ({:#b}) at PC {:#x} ({})", opcode, opcode, self.pc - 1, self.pc - 1),
        };
    }

    fn exec_prefixed_instruction(&mut self, opcode: u8, bus: &mut Bus)  {
        let real_opcode = self.read_opcode(bus);
        match real_opcode {
            // BIT 7,H.
            0x7C => {
                self.flag.z_zero = self.h >> 7 == 0;
                self.flag.n_substract = false;
                self.flag.h_half_carry = true;
            },

            // RL C.
            0x11 => rl!(self, c),

            _ => panic!("Unknown perfixed [{:#x} ({:#b})] opcode {:#x} ({:#b})", opcode, opcode, real_opcode, real_opcode),
        }
    }

    fn read_opcode(&mut self, bus: &Bus) -> u8 {
        self.read_byte(bus)
    }

    fn read_byte(&mut self, bus: &Bus) -> u8 {
        self.pc += 1;
        bus.read_byte((self.pc - 1) as usize)
    }

    fn read_low_high(&mut self, bus: &Bus) -> (u8, u8) {
        (self.read_byte(bus), self.read_byte(bus))
    }

    fn stack_push(&mut self, byte: u8, bus: &mut Bus) {
        bus.write_byte(self.sp as usize, byte);
        self.sp -= 1;

        if self.sp < STACK_BOTTOM {
            panic!("Stack pointer reached bottom: {:#x}.", self.sp);
        }
    }

    fn stack_pop(&mut self, bus: &Bus) -> u8 {
        self.sp += 1;
        if self.sp > STACK_TOP {
            panic!("Stack pointer reached top: {:#x}", self.sp);
        }

        // TODO too much "as usize", try to apply the From or Into trait
        bus.read_byte(self.sp as usize)
    }

}