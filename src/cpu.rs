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
            0b1100_1101 => {
                let (addr_hi, addr_lo) = u16_to_hi_lo(self.pc);
                // TODO review, http://www.devrs.com/gb/files/instr.txt does not mention SP adjustment here
                self.stack_push(addr_hi, bus);
                self.stack_push(addr_lo, bus);

                let (vlow, vhigh) = self.read_low_high(bus);
                self.pc = hi_lo_to_u16(vhigh, vlow);
            },

            // DEC B.
            0b0000_0101 => {
                self.b = if self.b == 0 { 0xFF } else { self.b - 1 };
                self.flag.z_zero = self.b == 0;
                self.flag.n_substract = true;
                self.flag.h_half_carry = self.b >> 3 & 1 == 0;
            },

            // INC C.
            0b000_1100 => {
                self.c = if self.c == 0xFF { 0 } else { self.c + 1 };
                // TODO verify if this is a conditional set or always. Now it's always.
                self.flag.z_zero = self.c == 0;
                self.flag.n_substract = false;
                // TODO verify if this a definite set or only when bit 3 == 1
                self.flag.h_half_carry = self.c >> 3 & 1 == 1;
            },
            // INC HL.
            0b0010_0011 => {
                let (hi, lo) = inc_dd(self.h, self.l);
                self.h = hi;
                self.l = lo;
            },

            // JR NZ,r8.
            0b0010_0000 => {
                let addr = self.read_byte(bus);
                if !self.flag.z_zero {
                    self.pc = ((self.pc as i16) + ((addr as i8) as i16)) as u16;
                }
            },

            // LD BC,d16.
            0b0000_0001 => {
                let (vlow, vhigh) = self.read_low_high(bus);
                self.c = vlow;
                self.b = vhigh;
            },
            // LD C,d8.
            0b0000_1110 => self.c = self.read_byte(bus),
            // LD DE,d16.
            0b0001_0001 => {
                let (vlow, vhigh) = self.read_low_high(bus);
                self.e = vlow;
                self.d = vhigh;
            },
            // LD A,(DE).
            0b0001_1010 => {
                let addr = (self.d as u16) << 8 | (self.e as u16);
                self.acc = bus.read_byte(addr as usize);
            }
            // LD HL,d16.
            0b0010_0001 => {
                let (vlow, vhigh) = self.read_low_high(bus);
                self.l = vlow;
                self.h = vhigh;
            },
            // LD SP,d16.
            0b0011_0001 => {
                let (vlow, vhigh) = self.read_low_high(bus);
                self.sp = (vhigh as u16) << 8 | (vlow as u16);
            },
            // LD (HL-),A.
            0b0011_0010 => {
                let mut addr = hi_lo_to_u16(self.h, self.l);
                bus.write_byte(addr as usize, self.acc);

                assert!(addr > 0, "Address reg HL is zero, cannot be decremented");
                addr -= 1;

                // TODO make it a func or macro.
                self.h = (addr >> 8) as u8;
                self.l = (addr & 0xFF) as u8;
            },
            // LD (HL+),A.
            0b0010_0010 => {
                let mut addr = hi_lo_to_u16(self.h, self.l);
                bus.write_byte(addr as usize, self.acc);

                assert!(addr < 0xFFFF, "Address reg HL is max (0xFFFF), cannot be incremented");
                addr += 1;

                // TODO make it a func or macro.
                self.h = (addr >> 8) as u8;
                self.l = (addr & 0xFF) as u8;
            },
            // LD A,d8.
            0b0011_1110 => self.acc = self.read_byte(bus),
            // LD C,A.
            0b0100_1111 => self.c = self.acc,
            // LD (HL),A.
            0b0111_0111 => {
                let addr = ((self.h as u16) << 8) | (self.l as u16);
                bus.write_byte(addr as usize, self.acc);
            },
            // LDH (n),A.
            0b1110_0000 => {
                let addr = self.read_byte(bus);
                bus.write_byte((0xFF00 + (addr as u16)) as usize, self.acc);
            },
            // LD (C),A.
            0b1110_0010 => {
                // let offs = self.read_byte(bus);
                // bus.write_byte((offs + self.c) as usize, self.acc);
                // There is some contradiciton here.
                // http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html mentions LD (C),A is a 2 byte op.
                // However it doesn't refer to a loaded byte as well as http://gbdev.gg8.se/wiki/articles/Gameboy_Bootstrap_ROM
                // says its a 1 byte op with fixed signing. We follow the latter now.
                bus.write_byte((0xFF00 + (self.c as u16)) as usize, self.acc);
            },
            // LD B,d8.
            0b0000_0110 => self.b = self.read_byte(bus),

            // POP BC.
            0b1100_0001 => {
                self.c = self.stack_pop(bus);
                self.b = self.stack_pop(bus);
            },

            // Prefix CB.
            0b1100_1011 => self.exec_prefixed_instruction(opcode, bus),

            // PUSH BC.
            0b1100_0101 => {
                let (b, c) = (self.b, self.c);
                self.stack_push(b, bus);
                self.stack_push(c, bus);
            },

            // RLA.
            0b0001_0111 => rl!(self, acc),

            // XOR B.
            0b1010_1000 => self.b ^= self.b,
            // XOR C.
            0b1010_1001 => self.c ^= self.c,
            // XOR D.
            0b1010_1010 => self.d ^= self.d,
            // XOR E.
            0b1010_1011 => self.e ^= self.e,
            // XOR H.
            0b1010_1100 => self.h ^= self.h,
            // XOR L.
            0b1010_1101 => self.l ^= self.l,
            // XOR A.
            0b1010_1111 => self.acc ^= self.acc,

            _ => panic!("Unknown opcode {:#x} ({:#b}) at PC {:#x} ({})", opcode, opcode, self.pc - 1, self.pc - 1),
        };
    }

    fn exec_prefixed_instruction(&mut self, opcode: u8, bus: &mut Bus)  {
        let real_opcode = self.read_opcode(bus);
        match real_opcode {
            // BIT 7,H.
            0b0111_1100 => {
                self.flag.z_zero = self.h >> 7 == 0;
                self.flag.n_substract = false;
                self.flag.h_half_carry = true;
            },

            // RL C.
            0b0001_0001 => rl!(self, c),

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