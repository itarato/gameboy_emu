use timer::Timer;
use bus::Bus;
use cpu;
use cpu::CPU;
use constants::*;

const TICK_DIV_REG: u64 = 255;
const TICK_SEQ_VIDEO_023: &'static str = "video_023";
const TICK_SEQ_VIDEO_1: &'static str = "video_1";

pub struct IO;

impl IO {
    pub fn new() -> IO {
        IO
    }

    pub fn init(&self, timer: &mut Timer) {
        timer.register_tick(TICK_DIV_REG);
        timer.register_tick_series(TICK_SEQ_VIDEO_023.to_owned(), vec![204, 80, 172]);
        timer.register_tick_series(TICK_SEQ_VIDEO_1.to_owned(), vec![67109, 4530]);
    }

    pub fn operate(&self, bus: &mut Bus) {
        if bus.timer.did_tick(TICK_DIV_REG) {
            let div = bus.read_byte(REG_DIV as usize);
            bus.write_byte(REG_DIV as usize, div.wrapping_add(1));
        }

        let mut stat_reg = bus.read_byte(REG_STAT as usize);
        match bus.timer.phase_of(TICK_SEQ_VIDEO_1.to_string()) {
            0 => {
                match bus.timer.phase_of(TICK_SEQ_VIDEO_023.to_string()) {
                    // 00: Entire Display Ram can be accessed
                    0 => {
                        stat_reg &= 0b1111_1100;
                    },
                    // 10: During Searching OAM-RAM
                    1 => {
                        stat_reg |= 0b0000_0010;
                        stat_reg &= 0b1111_1110;
                    },
                    // 11: During Transfering Data to LCD Driver
                    2 => {
                        stat_reg |= 0b0000_0011;
                    },
                    _ => panic!("Invalid Video phase."),
                };
            },
            // 01: During V-Blank
            1 => {
                stat_reg &= 0b1111_1101;
                stat_reg |= 0b0000_0001;
            },
            _ => panic!("Invalid Video phase."),
        };
        bus.write_byte(REG_STAT as usize, stat_reg);
    }
}
