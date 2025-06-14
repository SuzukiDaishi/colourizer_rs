# Colourizer Rs

## Building

After installing [Rust](https://rustup.rs/), you can compile Colourizer Rs as follows:

```shell
cargo xtask bundle colourizer_rs --release
```

## Testing with Python

You can validate the built VST3 using [Pedalboard](https://github.com/spotify/pedalboard):

```bash
pip install pedalboard numpy
python scripts/pedalboard_test.py
```
