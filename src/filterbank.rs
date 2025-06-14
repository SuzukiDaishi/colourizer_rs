// Filter bank for pitchmap-like effect

use std::collections::HashSet;

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
    active: HashSet<u8>,
}

impl FilterBank {
    /// Create a new filter bank. The provided scale lists the note names that
    /// should be audible.
    pub fn new(sample_rate: f32, scale: &[&str]) -> Self {
        let mut filters = Vec::new();
        let mut active = HashSet::new();
        for name in scale {
            if let Some(idx) = note_index(name) {
                active.insert(idx);
            }
        }

        // Piano range C0 (midi 12) .. B8 (midi 119)
        for midi in 12u8..=119u8 {
            let freq = 440.0_f32 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0);
            let idx = midi % 12;
            let filter = PeakFilter::new(freq, 12.0, 0.0, sample_rate);
            filters.push((idx, filter));
        }

        Self { filters, active }
    }

    /// Set the output scale using note names.
    pub fn set_scale(&mut self, scale: &[&str]) {
        self.active.clear();
        for name in scale {
            if let Some(idx) = note_index(name) {
                self.active.insert(idx);
            }
        }
    }

    /// Process a single sample through the filter bank.
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let mut sum = 0.0;
        for (idx, filter) in &mut self.filters {
            let out = filter.process(input);
            if self.active.contains(idx) {
                sum += out;
            }
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
    fn test_filterbank_scale() {
        let sr = 48000.0;
        let mut fb = FilterBank::new(sr, &["c", "g"]);
        assert!(fb.active.contains(&0));
        assert!(fb.active.contains(&7));
        assert!(!fb.active.contains(&1));

        fb.set_scale(&["d", "e"]);
        assert!(fb.active.contains(&2));
        assert!(fb.active.contains(&4));
        assert!(!fb.active.contains(&0));
    }

    #[test]
    fn test_filter_count() {
        let fb = FilterBank::new(44100.0, &["c"]);
        assert_eq!(fb.filters.len(), 108);
    }

    #[test]
    fn test_set_scale_duplicates() {
        let mut fb = FilterBank::new(44100.0, &["c", "c", "d"]);
        assert_eq!(fb.active.len(), 2);
        fb.set_scale(&["e", "e"]);
        assert_eq!(fb.active.len(), 1);
        assert!(fb.active.contains(&4));
    }

    #[test]
    fn test_process_sample_zero() {
        let mut fb = FilterBank::new(44100.0, &["c"]);
        assert_eq!(fb.process_sample(0.0), 0.0);
    }

    #[test]
    fn test_process_sample_no_active() {
        let mut fb = FilterBank::new(44100.0, &[]);
        assert_eq!(fb.process_sample(1.0), 0.0);
    }

    #[test]
    fn test_process_sample_single_note_nonzero() {
        let mut fb = FilterBank::new(44100.0, &["c"]);
        assert!(fb.process_sample(1.0) != 0.0);
    }

    #[test]
    fn test_set_scale_empty() {
        let mut fb = FilterBank::new(44100.0, &["c"]);
        fb.set_scale(&[]);
        assert!(fb.active.is_empty());
    }

    #[test]
    fn test_set_scale_invalid_names() {
        let mut fb = FilterBank::new(44100.0, &["x"]);
        assert!(fb.active.is_empty());
    }

    #[test]
    fn test_process_sample_after_scale_change() {
        let mut fb = FilterBank::new(44100.0, &["c"]);
        fb.process_sample(1.0);
        fb.set_scale(&["d"]);
        assert!(fb.process_sample(1.0) != 0.0);
    }
}

