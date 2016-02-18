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
            $_self.flag.h_half_carry = $_self.$reg >> 4 & 1 == 0;
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
            $_self.flag.h_half_carry = $_self.$reg >> 3 & 1 == 0;
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

macro_rules! cp {
    ($_self:expr, $cmp:expr) => (
        {
            let res = (($_self.acc as i8) - ($cmp as i8)) as u8;
            $_self.flag.z_zero = $_self.acc == $cmp;
            $_self.flag.n_substract = true;
            $_self.flag.h_half_carry = res >> 4 & 1 == 0;
            $_self.flag.c_carry = $_self.acc < $cmp;
        }
    )
}

macro_rules! call {
    ($_self:expr, $bus:expr, $cond:expr) => (
        {
            let (vlow, vhigh) = $_self.read_low_high($bus);

            if $cond {
                // Read address after data load so PC is set to next instruction.
                let (addr_hi, addr_lo) = u16_to_hi_lo($_self.pc);
                // TODO review, http://www.devrs.com/gb/files/instr.txt does not mention SP adjustment here
                $_self.stack_push(addr_hi, $bus);
                $_self.stack_push(addr_lo, $bus);

                $_self.pc = hi_lo_to_u16(vhigh, vlow);
            }
        }
    )
}

macro_rules! jr {
    ($_self:expr, $bus:expr, $cond:expr) => (
        {
            let addr = $_self.read_byte($bus);
            if !$_self.flag.z_zero {
                $_self.pc = (($_self.pc as i16) + ((addr as i8) as i16)) as u16;
            }
        }
    )
}

const STACK_TOP: u16 = 0xFFFE;
// TOOD verify it's true
const STACK_BOTTOM: u16 = 0xFF80;

// Address of Intterrupt flag.
const IF_ADDR: u16 = 0xFF0F;

// Memory mapping.
const ROM_BANK_ADDR_START: u16 = 0x0000;
const ROM_BANK_ADDR_END: u16 = 0x3FFF;

const DURATION_MAINS: [u8; 256] = [
    4, 12,  8,  8,  4,  4,  8,  4, 20,  8,  8,  8,  4,  4,  8,  4,
    4, 12,  8,  8,  4,  4,  8,  4, 12,  8,  8,  8,  4,  4,  8,  4,
    8, 12,  8,  8,  4,  4,  8,  4,  8,  8,  8,  8,  4,  4,  8,  4,
    8, 12,  8,  8, 12, 12, 12,  4,  8,  8,  8,  8,  4,  4,  8,  4,
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
    8,  8,  8,  8,  8,  8,  4,  8,  4,  4,  4,  4,  4,  4,  8,  4,
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
    4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
    8, 12, 12, 16, 12, 16,  8, 16,  8, 16, 12,  4, 12, 24,  8, 16,
    8, 12, 12,  0, 12, 16,  8, 16,  8, 16, 12,  0, 12,  0,  8, 16,
   12, 12,  8,  0,  0, 16,  8, 16, 16,  4, 16,  0,  0,  0,  8, 16,
   12, 12,  8,  4,  0, 16,  8, 16, 12,  8, 16,  4,  0,  0,  8, 16
];

const DURATION_MAIN_EXTRA: [u8; 256] = [
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    4,  0,  0,  0,  0,  0,  0,  0,  4,  0,  0,  0,  0,  0,  0,  0,
    4,  0,  0,  0,  0,  0,  0,  0,  4,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
   12,  0,  4,  0, 12,  0,  0,  0, 12,  0,  4,  0, 12,  0,  0,  0,
   12,  0,  4,  0, 12,  0,  0,  0, 12,  0,  4,  0, 12,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0
];

const DURATION_PREFIXED: [u8; 256] = [
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8,
    8,  8,  8,  8,  8,  8, 16,  8,  8,  8,  8,  8,  8,  8, 16,  8
];

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

    interrupts_enabled: bool,

    cycle: u64,
}

impl CPU {
    pub fn new() -> CPU {
        CPU::default()
    }

    pub fn reset(&mut self) {
        // Point to first instruction.
        self.pc = 0x0000;
        self.interrupts_enabled = true;
    }

    pub fn next_instruction(&mut self, bus: &mut Bus)  {
        let opcode = self.read_opcode(bus);
        println!("Read opcode {:#x} ({:#b}) at PC {:#x} ({})", opcode, opcode, self.pc - 1, self.pc - 1);

        match opcode {
            // CALL NZ,a16.
            0xC4 => call!(self, bus, !self.flag.z_zero),
            // CALL NC,a16.
            0xD4 => call!(self, bus, !self.flag.c_carry),
            // CALL Z,a16.
            0xCC => call!(self, bus, self.flag.z_zero),
            // CALL C,a16.
            0xDC => call!(self, bus, self.flag.c_carry),
            // CALL a16.
            0xCD => call!(self, bus, true),

            // CP B.
            0xB8 => cp!(self, self.b),
            // CP C.
            0xB9 => cp!(self, self.c),
            // CP D.
            0xBA => cp!(self, self.d),
            // CP E.
            0xBB => cp!(self, self.e),
            // CP H.
            0xBC => cp!(self, self.h),
            // CP L.
            0xBD => cp!(self, self.l),
            // CP A.
            0xBF => cp!(self, self.acc),

            // CP d8.
            0xFE => {
                let cmp = self.read_byte(bus);
                cp!(self, cmp);
            },
            // CP (HL).
            0xBE => {
                let cmp = bus.read_byte(hi_lo_to_u16(self.h, self.l) as usize);
                cp!(self, cmp);
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
            // DEC (HL).
            0x35 => {
                let addr = hi_lo_to_u16(self.h, self.l) as usize;
                let val = bus.read_byte(addr);
                bus.write_byte(addr, val - 1);
            },
            // DEC SP.
            0x3B => self.sp -= 1,

            // DI.
            // TODO check if it's a dedicated register or 0xFFFF (interrupt enable register).
            0xF3 => self.interrupts_enabled = false,

            // EI.
            // TODO check if it's a dedicated register or 0xFFFF (interrupt enable register).
            0xFB => self.interrupts_enabled = true,

            // HALT.
            0x76 => panic!("HALT."),

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
            // INC SP.
            0x33 => self.sp += 1,
            // INC (HL).
            0x34 => {
                let addr = hi_lo_to_u16(self.h, self.l) as usize;
                let val = bus.read_byte(addr);
                bus.write_byte(addr, val + 1);
            },

            // JR NZ,r8.
            0x20 => jr!(self, bus, !self.flag.z_zero),
            // JR NC,r8.
            0x30 => jr!(self, bus, !self.flag.c_carry),
            // JR r8.
            0x18 => jr!(self, bus, true),
            // JR Z,r8.
            0x28 => jr!(self, bus, self.flag.z_zero),
            // JR C,r8.
            0x38 => jr!(self, bus, self.flag.c_carry),

            // LD BC,d16.
            0x01 => {
                let (vlow, vhigh) = self.read_low_high(bus);
                self.c = vlow;
                self.b = vhigh;
            },
            // LD DE,d16.
            0x11 => {
                let (vlow, vhigh) = self.read_low_high(bus);
                self.e = vlow;
                self.d = vhigh;
            },
            // LD HL,d16.
            0x21 => {
                let (vlow, vhigh) = self.read_low_high(bus);
                self.l = vlow;
                self.h = vhigh;
            },
            // LD SP,d16.
            0x31 => {
                let (vlow, vhigh) = self.read_low_high(bus);
                self.sp = hi_lo_to_u16(vhigh, vlow);
            },

            // LD (BC),A.
            0x02 => bus.write_byte(hi_lo_to_u16(self.b, self.c) as usize, self.acc),
            // LD (DE),A.
            0x12 => bus.write_byte(hi_lo_to_u16(self.d, self.e) as usize, self.acc),
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


            // LD B,d8.
            0x06 => self.b = self.read_byte(bus),
            // LD D,d8.
            0x16 => self.d = self.read_byte(bus),
            // LD H,d8.
            0x26 => self.h = self.read_byte(bus),
            // LD (HL),d8.
            0x36 => {
                let val = self.read_byte(bus);
                bus.write_byte(hi_lo_to_u16(self.h, self.l) as usize, val)
            },
            // LD C,d8.
            0x0E => self.c = self.read_byte(bus),
            // LD E,d8.
            0x1E => self.e = self.read_byte(bus),
            // LD L,d8.
            0x2E => self.l = self.read_byte(bus),
            // LD A,d8.
            0x3E => self.acc = self.read_byte(bus),

            // LD A,(BC).
            0x0A => self.acc = bus.read_byte(hi_lo_to_u16(self.b, self.c) as usize),
            // LD A,(DE).
            0x1A => self.acc = bus.read_byte(hi_lo_to_u16(self.d, self.e) as usize),
            // LD A,(HL+).
            0x2A => {
                let mut addr = hi_lo_to_u16(self.h, self.l);
                self.acc = bus.read_byte(addr as usize);

                assert!(addr < 0xFFFF, "Address reg HL is max (0xFFFF), cannot be incremented");
                addr += 1;

                // TODO make it a func or macro.
                self.h = (addr >> 8) as u8;
                self.l = (addr & 0xFF) as u8;
            },
            // LD A,(HL-).
            0x3A => {
                let mut addr = hi_lo_to_u16(self.h, self.l);
                self.acc = bus.read_byte(addr as usize);

                assert!(addr > 0, "Address reg HL is zero, cannot be decremented");
                addr -= 1;

                // TODO make it a func or macro.
                self.h = (addr >> 8) as u8;
                self.l = (addr & 0xFF) as u8;
            },

            // LD B,B.
            0x40 => self.b = self.b,
            // LD B,C.
            0x41 => self.b = self.c,
            // LD B,D.
            0x42 => self.b = self.d,
            // LD B,E.
            0x43 => self.b = self.e,
            // LD B,H.
            0x44 => self.b = self.h,
            // LD B,L.
            0x45 => self.b = self.l,
            // LD B,(HL).
            0x46 => self.b = bus.read_byte(hi_lo_to_u16(self.h, self.l) as usize),
            // LD B,A.
            0x47 => self.b = self.acc,

            // LD C,B.
            0x48 => self.c = self.b,
            // LD C,C.
            0x49 => self.c = self.c,
            // LD C,D.
            0x4A => self.c = self.d,
            // LD C,E.
            0x4B => self.c = self.e,
            // LD C,H.
            0x4C => self.c = self.h,
            // LD C,L.
            0x4D => self.c = self.l,
            // LD C,(HL).
            0x4E => self.c = bus.read_byte(hi_lo_to_u16(self.h, self.l) as usize),
            // LD C,A.
            0x4F => self.c = self.acc,

            // LD D,B.
            0x50 => self.d = self.b,
            // LD D,C.
            0x51 => self.d = self.c,
            // LD D,D.
            0x52 => self.d = self.d,
            // LD D,E.
            0x53 => self.d = self.e,
            // LD D,H.
            0x54 => self.d = self.h,
            // LD D,L.
            0x55 => self.d = self.l,
            // LD D,(HL).
            0x56 => self.d = bus.read_byte(hi_lo_to_u16(self.h, self.l) as usize),
            // LD D,A.
            0x57 => self.d = self.acc,

            // LD E,B.
            0x58 => self.e = self.b,
            // LD E,C.
            0x59 => self.e = self.c,
            // LD E,D.
            0x5A => self.e = self.d,
            // LD E,E.
            0x5B => self.e = self.e,
            // LD E,H.
            0x5C => self.e = self.h,
            // LD E,L.
            0x5D => self.e = self.l,
            // LD E,(HL).
            0x5E => self.e = bus.read_byte(hi_lo_to_u16(self.h, self.l) as usize),
            // LD E,A.
            0x5F => self.e = self.acc,

            // LD H,B.
            0x60 => self.h = self.b,
            // LD H,C.
            0x61 => self.h = self.c,
            // LD H,D.
            0x62 => self.h = self.d,
            // LD H,E.
            0x63 => self.h = self.e,
            // LD H,H.
            0x64 => self.h = self.h,
            // LD H,L.
            0x65 => self.h = self.l,
            // LD H,(HL).
            0x66 => self.h = bus.read_byte(hi_lo_to_u16(self.h, self.l) as usize),
            // LD H,A.
            0x67 => self.h = self.acc,

            // LD L,B.
            0x68 => self.l = self.b,
            // LD L,C.
            0x69 => self.l = self.c,
            // LD L,D.
            0x6A => self.l = self.d,
            // LD L,E.
            0x6B => self.l = self.e,
            // LD L,H.
            0x6C => self.l = self.h,
            // LD L,L.
            0x6D => self.l = self.l,
            // LD L,(HL).
            0x6E => self.l = bus.read_byte(hi_lo_to_u16(self.h, self.l) as usize),
            // LD L,A.
            0x6F => self.l = self.acc,

            // LD (HL),B.
            0x70 => bus.write_byte(hi_lo_to_u16(self.h, self.l) as usize, self.b),
            // LD (HL),C.
            0x71 => bus.write_byte(hi_lo_to_u16(self.h, self.l) as usize, self.c),
            // LD (HL),D.
            0x72 => bus.write_byte(hi_lo_to_u16(self.h, self.l) as usize, self.d),
            // LD (HL),E.
            0x73 => bus.write_byte(hi_lo_to_u16(self.h, self.l) as usize, self.e),
            // LD (HL),H.
            0x74 => bus.write_byte(hi_lo_to_u16(self.h, self.l) as usize, self.h),
            // LD (HL),L.
            0x75 => bus.write_byte(hi_lo_to_u16(self.h, self.l) as usize, self.l),
            // LD (HL),A.
            0x77 => bus.write_byte(hi_lo_to_u16(self.h, self.l) as usize, self.acc),

            // LD A,B.
            0x78 => self.acc = self.b,
            // LD A,C.
            0x79 => self.acc = self.c,
            // LD A,D.
            0x7A => self.acc = self.d,
            // LD A,E.
            0x7B => self.acc = self.e,
            // LD A,H.
            0x7C => self.acc = self.h,
            // LD A,L.
            0x7D => self.acc = self.l,
            // LD A,(HL).
            0x7E => self.acc = bus.read_byte(hi_lo_to_u16(self.h, self.l) as usize),
            // LD A,A.
            0x7F => self.acc = self.acc,

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
            // LD (a16),A.
            0xEA => {
                let (lo, hi) = self.read_low_high(bus);
                let addr = hi_lo_to_u16(hi, lo);
                bus.write_byte(addr as usize, self.acc);
            },
            // LDH A,(a8).
            0xF0 => {
                let offs = self.read_byte(bus);
                self.acc = bus.read_byte((0xFF00 | (offs as u16)) as usize);
            },

            // NOP.
            0x00 => { },

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

            // RET.
            0xC9 => {
                let (vlow, vhigh) = (self.stack_pop(bus), self.stack_pop(bus));
                self.pc = hi_lo_to_u16(vhigh, vlow);
            },

            // RLCA.
            0x07 => {
                self.flag.c_carry = self.acc >> 7 == 1;
                self.acc = self.acc << 1 | self.acc >> 7;
                self.flag.z_zero = self.acc == 0;
                self.flag.n_substract = false;
                self.flag.h_half_carry = false;
            },
            // RLA.
            0x17 => rl!(self, acc),

            // STOP.
            /* Halt until button pressed. Might be a better way to simulate. */
            0x10 => {
                let _ = self.read_byte(bus);
                panic!("STOP instruction called.")
            },

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

        self.inc_cycle(DURATION_MAINS[opcode as usize] as u64);
        // TODO handle higher duration time for the selected operations.

        self.handle_timing();
    }

    pub fn check_interrupt(&self, bus: &mut Bus) {
        if !self.interrupts_enabled {
            return;
        }

        let int_byte = bus.read_byte(IF_ADDR as usize);

        // Bit 0: V-Blank Interrupt Request (INT 40h)  (1=Request)
        if int_byte >> 1 == 1 {
            panic!("Interrupt needs to be handled: V-Blank Interrupt Request");
        }

        // Bit 1: LCD STAT Interrupt Request (INT 48h)  (1=Request)
        if int_byte >> 1 & 1 == 1 {
            panic!("Interrupt needs to be handled: LCD STAT Interrupt Request");
        }

        // Bit 2: Timer Interrupt Request (INT 50h)  (1=Request)
        if int_byte >> 2 & 1 == 1 {
            panic!("Interrupt needs to be handled: Timer Interrupt Request");
        }

        // Bit 3: Serial Interrupt Request (INT 58h)  (1=Request)
        if int_byte >> 3 & 1 == 1 {
            panic!("Interrupt needs to be handled: Serial Interrupt Request");
        }

        // Bit 4: Joypad Interrupt Request (INT 60h)  (1=Request)
        if int_byte >> 4 & 1 == 1 {
            panic!("Interrupt needs to be handled: Joypad Interrupt Request");
        }
    }

    fn inc_cycle(&mut self, n: u64) {
        self.cycle += n;
    }

    fn handle_timing(&self) {
        // TODO
    }

    fn exec_prefixed_instruction(&mut self, opcode: u8, bus: &mut Bus)  {
        let real_opcode = self.read_opcode(bus);
        match real_opcode {
            // RLC B.
            // 0x00 => { },
            // RLC C.
            // 0x01 => { },
            // RLC D.
            // 0x02 => { },
            // RLC E.
            // 0x03 => { },
            // RLC H.
            // 0x04 => { },
            // RLC L.
            // 0x05 => { },
            // RLC (HL).
            // 0x06 => { },
            // RLC A.
            // 0x07 => { },
            // RRC B.
            // 0x08 => { },
            // RRC C.
            // 0x09 => { },
            // RRC D.
            // 0x0A => { },
            // RRC E.
            // 0x0B => { },
            // RRC H.
            // 0x0C => { },
            // RRC L.
            // 0x0D => { },
            // RRC (HL).
            // 0x0E => { },
            // RRC A.
            // 0x0F => { },
            // RL B.
            // 0x10 => { },
            // RL C.
            0x11 => rl!(self, c),
            // RL D.
            // 0x12 => { },
            // RL E.
            // 0x13 => { },
            // RL H.
            // 0x14 => { },
            // RL L.
            // 0x15 => { },
            // RL (HL).
            // 0x16 => { },
            // RL A.
            // 0x17 => { },
            // RR B.
            // 0x18 => { },
            // RR C.
            // 0x19 => { },
            // RR D.
            // 0x1A => { },
            // RR E.
            // 0x1B => { },
            // RR H.
            // 0x1C => { },
            // RR L.
            // 0x1D => { },
            // RR (HL).
            // 0x1E => { },
            // RR A.
            // 0x1F => { },
            // SLA B.
            // 0x20 => { },
            // SLA C.
            // 0x21 => { },
            // SLA D.
            // 0x22 => { },
            // SLA E.
            // 0x23 => { },
            // SLA H.
            // 0x24 => { },
            // SLA L.
            // 0x25 => { },
            // SLA (HL).
            // 0x26 => { },
            // SLA A.
            // 0x27 => { },
            // SRA B.
            // 0x28 => { },
            // SRA C.
            // 0x29 => { },
            // SRA D.
            // 0x2A => { },
            // SRA E.
            // 0x2B => { },
            // SRA H.
            // 0x2C => { },
            // SRA L.
            // 0x2D => { },
            // SRA (HL).
            // 0x2E => { },
            // SRA A.
            // 0x2F => { },
            // SWAP B.
            // 0x30 => { },
            // SWAP C.
            // 0x31 => { },
            // SWAP D.
            // 0x32 => { },
            // SWAP E.
            // 0x33 => { },
            // SWAP H.
            // 0x34 => { },
            // SWAP L.
            // 0x35 => { },
            // SWAP (HL).
            // 0x36 => { },
            // SWAP A.
            // 0x37 => { },
            // SRL B.
            // 0x38 => { },
            // SRL C.
            // 0x39 => { },
            // SRL D.
            // 0x3A => { },
            // SRL E.
            // 0x3B => { },
            // SRL H.
            // 0x3C => { },
            // SRL L.
            // 0x3D => { },
            // SRL (HL).
            // 0x3E => { },
            // SRL A.
            // 0x3F => { },
            // BIT 0,B.
            // 0x40 => { },
            // BIT 0,C.
            // 0x41 => { },
            // BIT 0,D.
            // 0x42 => { },
            // BIT 0,E.
            // 0x43 => { },
            // BIT 0,H.
            // 0x44 => { },
            // BIT 0,L.
            // 0x45 => { },
            // BIT 0,(HL).
            // 0x46 => { },
            // BIT 0,A.
            // 0x47 => { },
            // BIT 1,B.
            // 0x48 => { },
            // BIT 1,C.
            // 0x49 => { },
            // BIT 1,D.
            // 0x4A => { },
            // BIT 1,E.
            // 0x4B => { },
            // BIT 1,H.
            // 0x4C => { },
            // BIT 1,L.
            // 0x4D => { },
            // BIT 1,(HL).
            // 0x4E => { },
            // BIT 1,A.
            // 0x4F => { },
            // BIT 2,B.
            // 0x50 => { },
            // BIT 2,C.
            // 0x51 => { },
            // BIT 2,D.
            // 0x52 => { },
            // BIT 2,E.
            // 0x53 => { },
            // BIT 2,H.
            // 0x54 => { },
            // BIT 2,L.
            // 0x55 => { },
            // BIT 2,(HL).
            // 0x56 => { },
            // BIT 2,A.
            // 0x57 => { },
            // BIT 3,B.
            // 0x58 => { },
            // BIT 3,C.
            // 0x59 => { },
            // BIT 3,D.
            // 0x5A => { },
            // BIT 3,E.
            // 0x5B => { },
            // BIT 3,H.
            // 0x5C => { },
            // BIT 3,L.
            // 0x5D => { },
            // BIT 3,(HL).
            // 0x5E => { },
            // BIT 3,A.
            // 0x5F => { },
            // BIT 4,B.
            // 0x60 => { },
            // BIT 4,C.
            // 0x61 => { },
            // BIT 4,D.
            // 0x62 => { },
            // BIT 4,E.
            // 0x63 => { },
            // BIT 4,H.
            // 0x64 => { },
            // BIT 4,L.
            // 0x65 => { },
            // BIT 4,(HL).
            // 0x66 => { },
            // BIT 4,A.
            // 0x67 => { },
            // BIT 5,B.
            // 0x68 => { },
            // BIT 5,C.
            // 0x69 => { },
            // BIT 5,D.
            // 0x6A => { },
            // BIT 5,E.
            // 0x6B => { },
            // BIT 5,H.
            // 0x6C => { },
            // BIT 5,L.
            // 0x6D => { },
            // BIT 5,(HL).
            // 0x6E => { },
            // BIT 5,A.
            // 0x6F => { },
            // BIT 6,B.
            // 0x70 => { },
            // BIT 6,C.
            // 0x71 => { },
            // BIT 6,D.
            // 0x72 => { },
            // BIT 6,E.
            // 0x73 => { },
            // BIT 6,H.
            // 0x74 => { },
            // BIT 6,L.
            // 0x75 => { },
            // BIT 6,(HL).
            // 0x76 => { },
            // BIT 6,A.
            // 0x77 => { },
            // BIT 7,B.
            // 0x78 => { },
            // BIT 7,C.
            // 0x79 => { },
            // BIT 7,D.
            // 0x7A => { },
            // BIT 7,E.
            // 0x7B => { },
            // BIT 7,H.
            0x7C => {
                self.flag.z_zero = self.h >> 7 == 0;
                self.flag.n_substract = false;
                self.flag.h_half_carry = true;
            },
            // BIT 7,L.
            // 0x7D => { },
            // BIT 7,(HL).
            // 0x7E => { },
            // BIT 7,A.
            // 0x7F => { },
            // RES 0,B.
            // 0x80 => { },
            // RES 0,C.
            // 0x81 => { },
            // RES 0,D.
            // 0x82 => { },
            // RES 0,E.
            // 0x83 => { },
            // RES 0,H.
            // 0x84 => { },
            // RES 0,L.
            // 0x85 => { },
            // RES 0,(HL).
            // 0x86 => { },
            // RES 0,A.
            // 0x87 => { },
            // RES 1,B.
            // 0x88 => { },
            // RES 1,C.
            // 0x89 => { },
            // RES 1,D.
            // 0x8A => { },
            // RES 1,E.
            // 0x8B => { },
            // RES 1,H.
            // 0x8C => { },
            // RES 1,L.
            // 0x8D => { },
            // RES 1,(HL).
            // 0x8E => { },
            // RES 1,A.
            // 0x8F => { },
            // RES 2,B.
            // 0x90 => { },
            // RES 2,C.
            // 0x91 => { },
            // RES 2,D.
            // 0x92 => { },
            // RES 2,E.
            // 0x93 => { },
            // RES 2,H.
            // 0x94 => { },
            // RES 2,L.
            // 0x95 => { },
            // RES 2,(HL).
            // 0x96 => { },
            // RES 2,A.
            // 0x97 => { },
            // RES 3,B.
            // 0x98 => { },
            // RES 3,C.
            // 0x99 => { },
            // RES 3,D.
            // 0x9A => { },
            // RES 3,E.
            // 0x9B => { },
            // RES 3,H.
            // 0x9C => { },
            // RES 3,L.
            // 0x9D => { },
            // RES 3,(HL).
            // 0x9E => { },
            // RES 3,A.
            // 0x9F => { },
            // RES 4,B.
            // 0xA0 => { },
            // RES 4,C.
            // 0xA1 => { },
            // RES 4,D.
            // 0xA2 => { },
            // RES 4,E.
            // 0xA3 => { },
            // RES 4,H.
            // 0xA4 => { },
            // RES 4,L.
            // 0xA5 => { },
            // RES 4,(HL).
            // 0xA6 => { },
            // RES 4,A.
            // 0xA7 => { },
            // RES 5,B.
            // 0xA8 => { },
            // RES 5,C.
            // 0xA9 => { },
            // RES 5,D.
            // 0xAA => { },
            // RES 5,E.
            // 0xAB => { },
            // RES 5,H.
            // 0xAC => { },
            // RES 5,L.
            // 0xAD => { },
            // RES 5,(HL).
            // 0xAE => { },
            // RES 5,A.
            // 0xAF => { },
            // RES 6,B.
            // 0xB0 => { },
            // RES 6,C.
            // 0xB1 => { },
            // RES 6,D.
            // 0xB2 => { },
            // RES 6,E.
            // 0xB3 => { },
            // RES 6,H.
            // 0xB4 => { },
            // RES 6,L.
            // 0xB5 => { },
            // RES 6,(HL).
            // 0xB6 => { },
            // RES 6,A.
            // 0xB7 => { },
            // RES 7,B.
            // 0xB8 => { },
            // RES 7,C.
            // 0xB9 => { },
            // RES 7,D.
            // 0xBA => { },
            // RES 7,E.
            // 0xBB => { },
            // RES 7,H.
            // 0xBC => { },
            // RES 7,L.
            // 0xBD => { },
            // RES 7,(HL).
            // 0xBE => { },
            // RES 7,A.
            // 0xBF => { },
            // SET 0,B.
            // 0xC0 => { },
            // SET 0,C.
            // 0xC1 => { },
            // SET 0,D.
            // 0xC2 => { },
            // SET 0,E.
            // 0xC3 => { },
            // SET 0,H.
            // 0xC4 => { },
            // SET 0,L.
            // 0xC5 => { },
            // SET 0,(HL).
            // 0xC6 => { },
            // SET 0,A.
            // 0xC7 => { },
            // SET 1,B.
            // 0xC8 => { },
            // SET 1,C.
            // 0xC9 => { },
            // SET 1,D.
            // 0xCA => { },
            // SET 1,E.
            // 0xCB => { },
            // SET 1,H.
            // 0xCC => { },
            // SET 1,L.
            // 0xCD => { },
            // SET 1,(HL).
            // 0xCE => { },
            // SET 1,A.
            // 0xCF => { },
            // SET 2,B.
            // 0xD0 => { },
            // SET 2,C.
            // 0xD1 => { },
            // SET 2,D.
            // 0xD2 => { },
            // SET 2,E.
            // 0xD3 => { },
            // SET 2,H.
            // 0xD4 => { },
            // SET 2,L.
            // 0xD5 => { },
            // SET 2,(HL).
            // 0xD6 => { },
            // SET 2,A.
            // 0xD7 => { },
            // SET 3,B.
            // 0xD8 => { },
            // SET 3,C.
            // 0xD9 => { },
            // SET 3,D.
            // 0xDA => { },
            // SET 3,E.
            // 0xDB => { },
            // SET 3,H.
            // 0xDC => { },
            // SET 3,L.
            // 0xDD => { },
            // SET 3,(HL).
            // 0xDE => { },
            // SET 3,A.
            // 0xDF => { },
            // SET 4,B.
            // 0xE0 => { },
            // SET 4,C.
            // 0xE1 => { },
            // SET 4,D.
            // 0xE2 => { },
            // SET 4,E.
            // 0xE3 => { },
            // SET 4,H.
            // 0xE4 => { },
            // SET 4,L.
            // 0xE5 => { },
            // SET 4,(HL).
            // 0xE6 => { },
            // SET 4,A.
            // 0xE7 => { },
            // SET 5,B.
            // 0xE8 => { },
            // SET 5,C.
            // 0xE9 => { },
            // SET 5,D.
            // 0xEA => { },
            // SET 5,E.
            // 0xEB => { },
            // SET 5,H.
            // 0xEC => { },
            // SET 5,L.
            // 0xED => { },
            // SET 5,(HL).
            // 0xEE => { },
            // SET 5,A.
            // 0xEF => { },
            // SET 6,B.
            // 0xF0 => { },
            // SET 6,C.
            // 0xF1 => { },
            // SET 6,D.
            // 0xF2 => { },
            // SET 6,E.
            // 0xF3 => { },
            // SET 6,H.
            // 0xF4 => { },
            // SET 6,L.
            // 0xF5 => { },
            // SET 6,(HL).
            // 0xF6 => { },
            // SET 6,A.
            // 0xF7 => { },
            // SET 7,B.
            // 0xF8 => { },
            // SET 7,C.
            // 0xF9 => { },
            // SET 7,D.
            // 0xFA => { },
            // SET 7,E.
            // 0xFB => { },
            // SET 7,H.
            // 0xFC => { },
            // SET 7,L.
            // 0xFD => { },
            // SET 7,(HL).
            // 0xFE => { },
            // SET 7,A.
            // 0xFF => { },

            _ => panic!("Unknown perfixed [{:#x} ({:#b})] opcode {:#x} ({:#b})", opcode, opcode, real_opcode, real_opcode),
        };

        self.inc_cycle(DURATION_PREFIXED[real_opcode as usize] as u64);
    }

    fn read_opcode(&mut self, bus: &Bus) -> u8 {
        self.read_byte(bus)
    }

    fn read_byte(&mut self, bus: &Bus) -> u8 {
        let addr = self.pc as usize;

        self.pc += 1;
        // PC is virtually mapped to the ROM bank memory section.
        if self.pc > ROM_BANK_ADDR_END {
            println!("PC overflow to {:#x}", self.pc);
            self.pc -= ROM_BANK_ADDR_END - ROM_BANK_ADDR_START + 1;
        }

        bus.read_byte(addr)
    }

    fn read_low_high(&mut self, bus: &Bus) -> (u8, u8) {
        (self.read_byte(bus), self.read_byte(bus))
    }

    fn stack_push(&mut self, byte: u8, bus: &mut Bus) {
        // println!("Write to STACK[{:#x}] == {:#x}", self.sp, byte);
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

        // println!("Read from STACK[{:#x}]", self.sp);
        // TODO too much "as usize", try to apply the From or Into trait
        bus.read_byte(self.sp as usize)
    }

}