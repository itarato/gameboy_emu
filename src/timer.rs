use std::collections::HashMap;

#[derive(Debug, Default)]
struct Ticker {
    cycle: u64,
    did_tick: bool,
}

#[derive(Debug)]
struct SequenceTicker {
    lengths: Vec<u64>,
    current_phase: usize,
    current_cycle: u64,
}

#[derive(Default)]
pub struct Timer {
    ticks: HashMap<u64, Ticker>,
    sequences: HashMap<String, SequenceTicker>,
}

impl Timer {
    pub fn new() -> Timer {
        Timer::default()
    }

    pub fn register_tick(&mut self, cycle: u64) {
        self.ticks.insert(cycle, Ticker::default());
    }

    pub fn register_tick_series(&mut self, name: String, lengths: Vec<u64>) {
        let series_ticker = SequenceTicker {
            lengths: lengths,
            current_phase: 0,
            current_cycle: 0,
        };
        self.sequences.insert(name, series_ticker);
    }

    pub fn phase_of(&self, name: String) -> usize {
        self.sequences.get(&name).unwrap().current_phase
    }

    pub fn inc(&mut self, curr_cycles: u64) {
        for (cycle, ticker) in self.ticks.iter_mut() {
            ticker.cycle += curr_cycles;
            if ticker.cycle >= *cycle {
                ticker.cycle = ticker.cycle - cycle + 1;
                ticker.did_tick = true;
                println!("TICK {:?}", cycle);
            }
        }

        for (_, seq) in self.sequences.iter_mut() {
            seq.current_cycle += curr_cycles;
            while seq.current_cycle > seq.lengths[seq.current_phase] {
                seq.current_cycle -= seq.lengths[seq.current_phase];
                seq.current_phase += 1;
                if seq.current_phase >= seq.lengths.len() {
                    seq.current_phase = 0;
                }
            }
        }
    }

    pub fn did_tick(&mut self, cycle: u64) -> bool {
        let mut ticker = self.ticks.get_mut(&cycle).unwrap();
        let did_tick = ticker.did_tick;
        ticker.did_tick = false;
        did_tick
    }
}
