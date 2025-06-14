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

After bundling the plugin, you can verify its behaviour with [Pedalboard](https://github.com/spotify/pedalboard).
Use [`uv`](https://github.com/astral-sh/uv) to create a virtual environment and
install the dependencies listed in `requirements.txt`:

```shell
uv venv
uv pip install -r requirements.txt
uv run python pedalboard_test.py
```

The script generates `test_results.md` and `test_results.json` with detailed information about multiple sample rates, mono/stereo inputs and gain settings. It also records the time required for each processing run so you can gauge performance.
