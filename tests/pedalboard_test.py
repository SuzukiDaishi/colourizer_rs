import json
import time
from pathlib import Path

RESULTS_DIR = Path(__file__).parent

import numpy as np
import pytest
try:
    from pedalboard import Pedalboard, load_plugin
except Exception:  # pragma: no cover - optional dependency
    pytest.skip("pedalboard not available", allow_module_level=True)

PLUGIN_PATH = "target/bundled/Colourizer Rs.vst3"


def _set_parameter(plugin, name: str, value: float) -> None:
    """Best effort helper to update a parameter by name."""
    if hasattr(plugin, "set_parameter"):
        try:
            plugin.set_parameter(name, value)
            return
        except Exception:
            pass
    # Fallback to searching the parameters list
    if hasattr(plugin, "parameters"):
        for p in plugin.parameters:
            if getattr(p, "name", None) == name:
                p.value = value
                break


def run_case(sample_rate: int, channels: int, freq: float, gain: float, mode: str, mix: float) -> dict:
    plugin = load_plugin(PLUGIN_PATH)
    _set_parameter(plugin, "Gain", gain)
    _set_parameter(plugin, "Processing Mode", mode)
    _set_parameter(plugin, "Dry/Wet", mix)
    board = Pedalboard([plugin])

    n = sample_rate
    t = np.arange(n) / sample_rate
    sine = np.sin(2 * np.pi * freq * t).astype(np.float32)
    if channels == 1:
        audio = sine[:, None]
    else:
        audio = np.stack([sine] * channels, axis=1)

    start = time.perf_counter()
    processed = board(audio, sample_rate)
    elapsed = time.perf_counter() - start

    return {
        "sample_rate": sample_rate,
        "channels": channels,
        "frequency": freq,
        "gain": gain,
        "shape": list(processed.shape),
        "min": float(processed.min()),
        "max": float(processed.max()),
        "elapsed_sec": elapsed,
        "mode": mode,
        "mix": mix,
    }


def main() -> None:
    results = []
    for sr in (44100, 48000):
        for ch in (1, 2, 6):
            for mode in ("Mono", "Multi"):
                for gain in (0.5, 1.0):
                    for mix in (0.0, 1.0):
                        for freq in (220.0, 440.0, 880.0, 1760.0):
                            results.append(run_case(sr, ch, freq, gain, mode, mix))

    (RESULTS_DIR / "test_results.md").write_text(
        "# Python pedalboard test results\n\n"
    )
    with open(RESULTS_DIR / "test_results.md", "a") as f:
        for r in results:
            f.write(
                f"- sr={r['sample_rate']} ch={r['channels']} mode={r['mode']} "
                f"gain={r['gain']} mix={r['mix']} f={r['frequency']} "
                f"shape={r['shape']} min={r['min']:.3f} max={r['max']:.3f} "
                f"elapsed={r['elapsed_sec']:.4f}s\n"
            )
    with open(RESULTS_DIR / "test_results.json", "w") as f:
        json.dump(results, f, indent=2)


if __name__ == "__main__":
    main()
