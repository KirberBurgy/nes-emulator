pub struct APUMixer {
    pulse_table: [f32; 31],
    tnd_table:   [[[f32; 128]; 16]; 16],
}

impl APUMixer {
    pub fn new() -> Self {
        let mut pulse_table = [0.0; 31];
        for i in 1..31 {
            pulse_table[i] = 95.88 / (8128.0 / (i as f32) + 100.0);
        }

        let mut tnd_table = [[[0.0; 128]; 16]; 16];
        
        for t in 0..16 {
            for n in 0..16 {
                for d in 0..128 {
                    if t > 0 || n > 0 || d > 0 {
                        let denom = (t as f32 / 8227.0) + (n as f32 / 12241.0) + (d as f32 / 22638.0);
                        tnd_table[t][n][d] = 159.79 / ((1.0 / denom) + 100.0);
                    }
                }
            }
        }

        APUMixer { pulse_table, tnd_table }
    }

    pub fn mix(&self, p1: u8, p2: u8, t: u8, n: u8, d: u8) -> f32 {
        let pulse_out = self.pulse_table[(p1 + p2) as usize];
        let tnd_out = self.tnd_table[t as usize][n as usize][d as usize];
        
        pulse_out + tnd_out
    }
}