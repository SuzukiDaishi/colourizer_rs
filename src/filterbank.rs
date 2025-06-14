// Filter bank for pitchmap-like effect

/// Simple peaking biquad filter.
/// Coefficients are calculated for unity gain or attenuation.
#[derive(Clone, Copy)]
struct PeakFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl PeakFilter {
    /// Create a new peaking filter with the given parameters.
    fn new(freq: f32, q: f32, gain_db: f32, sample_rate: f32) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let w0 = 2.0 * std::f32::consts::PI * freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * w0.cos();
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// Process a single sample through the filter.
    fn process(&mut self, input: f32) -> f32 {
        let out = self.b0 * input + self.z1;
        self.z1 = self.b1 * input - self.a1 * out + self.z2;
        self.z2 = self.b2 * input - self.a2 * out;
        out
    }
}

/// Convert a note name to a semitone index from C.
fn note_index(name: &str) -> Option<u8> {
    match name.to_ascii_lowercase().as_str() {
        "c" => Some(0),
        "c#" | "db" => Some(1),
        "d" => Some(2),
        "d#" | "eb" => Some(3),
        "e" => Some(4),
        "f" => Some(5),
        "f#" | "gb" => Some(6),
        "g" => Some(7),
        "g#" | "ab" => Some(8),
        "a" => Some(9),
        "a#" | "bb" => Some(10),
        "b" | "cb" => Some(11),
        _ => None,
    }
}

/// Filter bank with a peak filter for each note from C0 to B8.
pub struct FilterBank {
    filters: Vec<(u8, PeakFilter)>,
    gains: [f32; 12],
}

impl FilterBank {
    /// Create a new filter bank. The provided scale lists the note names that
    /// should be audible.
    pub fn new(sample_rate: f32) -> Self {
        let mut filters = Vec::new();

        // Piano range C0 (midi 12) .. B8 (midi 119)
        for midi in 12u8..=119u8 {
            let freq = 440.0_f32 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0);
            let idx = midi % 12;
            let filter = PeakFilter::new(freq, 12.0, 0.0, sample_rate);
            filters.push((idx, filter));
        }

        Self {
            filters,
            gains: [1.0; 12],
        }
    }

    /// Update the per-note gains. Expects an array of 12 values for C..B.
    pub fn set_gains(&mut self, gains: [f32; 12]) {
        self.gains = gains;
    }

    /// Process a single sample through the filter bank.
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let mut sum = 0.0;
        for (idx, filter) in &mut self.filters {
            let out = filter.process(input);
            sum += out * self.gains[*idx as usize];
        }
        sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_index() {
        assert_eq!(note_index("c"), Some(0));
        assert_eq!(note_index("c#"), Some(1));
        assert_eq!(note_index("db"), Some(1));
        assert_eq!(note_index("g"), Some(7));
        assert_eq!(note_index("ab"), Some(8));
        assert_eq!(note_index("bb"), Some(10));
        assert_eq!(note_index("h"), None);
    }

    #[test]
    fn test_note_index_case_insensitive() {
        assert_eq!(note_index("C"), Some(0));
        assert_eq!(note_index("F#"), Some(6));
        assert_eq!(note_index("Gb"), Some(6));
    }

    #[test]
    fn test_note_index_invalid() {
        assert_eq!(note_index("e#"), None);
        assert_eq!(note_index("r"), None);
    }

    #[test]
    fn test_filterbank_gains() {
        let sr = 48_000.0;
        let mut fb = FilterBank::new(sr);
        let gains = [1.0, 0.5, 0.0, 0.5, 1.0, 0.5, 0.0, 0.5, 1.0, 0.5, 0.0, 0.5];
        fb.set_gains(gains);
        assert_eq!(fb.gains[0], 1.0);
        assert_eq!(fb.gains[2], 0.0);
        assert_eq!(fb.gains[11], 0.5);
    }

    #[test]
    fn test_filter_count() {
        let fb = FilterBank::new(44100.0);
        assert_eq!(fb.filters.len(), 108);
    }

    #[test]
    fn test_set_gains_updates() {
        let mut fb = FilterBank::new(44100.0);
        let mut gains = [1.0_f32; 12];
        gains[0] = 0.2;
        fb.set_gains(gains);
        assert_eq!(fb.gains[0], 0.2);
    }

    #[test]
    fn test_process_sample_zero() {
        let mut fb = FilterBank::new(44100.0);
        assert_eq!(fb.process_sample(0.0), 0.0);
    }

    #[test]
    fn test_process_sample_no_active() {
        let mut fb = FilterBank::new(44100.0);
        fb.set_gains([0.0; 12]);
        assert_eq!(fb.process_sample(1.0), 0.0);
    }

    #[test]
    fn test_process_sample_single_note_nonzero() {
        let mut fb = FilterBank::new(44100.0);
        assert!(fb.process_sample(1.0) != 0.0);
    }
}
