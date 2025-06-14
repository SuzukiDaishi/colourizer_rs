# Test Plan

## Rust unit tests (FilterBank)
- note index recognizes sharps and flats
- note index is case insensitive
- invalid note names return None
- filterbank initializes with active scale
- filter count covers piano range
- duplicate scale entries are ignored
- `set_scale` replaces active notes
- processing zero input yields zero output
- no active notes produce silence
- processing after scale change works

## VST plugin tests via Pedalboard
- plugin loads successfully as VST3
- processes stereo audio at 44.1kHz
- processes stereo audio at 48kHz
- processes stereo audio at 96kHz
- handles multiple sine frequencies
- output is bounded between -1.0 and 1.0
- runtime measured for each case
- results recorded to `test_results.md`
- no crashes during processing
- bundle is located at `target/bundled`
