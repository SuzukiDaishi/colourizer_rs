# Test Plan

## Rust unit tests (FilterBank)
Run with `cargo test --quiet`.

- `note_index` resolves natural notes, sharps and flats
- `note_index` handles upper and lower case names
- invalid names return `None`
- creating a new `FilterBank` yields 108 filters (C0..B8)
- `set_gains` correctly updates the internal gain array
- processing zero input returns zero output
- zero gains result in silence
- processing with default gains produces non-zero output

## VST plugin tests via Pedalboard
Use `uv` to create a virtual environment and install `numpy` and
`pedalboard`:

```shell
uv venv
uv pip install -r requirements.txt
```

Tests are executed with `uv run python pedalboard_test.py`.

- plugin loads successfully from `target/bundled/Colourizer Rs.vst3`
- sample rates 44.1kHz, 48kHz and 96kHz
- both mono (1ch) and stereo (2ch) inputs are processed
- input gain parameter is swept between 0.5 and 1.0
- four sine wave frequencies (220Hz, 440Hz, 880Hz, 1760Hz)
- min/max levels and runtime for each run are logged
- results are written to `test_results.md` and `test_results.json`
