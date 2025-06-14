# Colourizer Rs

Colourizer Rs is a simple VST3/CLAP audio effect written in Rust using [NIH-plug](https://github.com/robbert-vdh/nih-plug). It filters incoming audio through a bank of peaking filters based on musical note names.

## Building

Make sure Rust and `cargo` are installed. To compile and bundle the plugin as a VST3 file, run:

```shell
cargo xtask bundle colourizer_rs --release
```

The resulting `Colourizer Rs.vst3` bundle will be placed in `target/bundled`.

## Testing with Python

Tests can be executed using `cargo`:

```shell
cargo test --all --no-fail-fast
```

After bundling the plugin, you can verify its behaviour with [Pedalboard](https://github.com/spotify/pedalboard). First install the package and then run the included script:

```shell
pip install pedalboard
python pedalboard_test.py
```

The script generates `test_results.md` with information about multiple sample rates and frequencies. It also records the time required for each processing run so you can gauge performance.
