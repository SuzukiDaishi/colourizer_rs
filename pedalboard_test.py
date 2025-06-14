import json
import time
from pathlib import Path

import numpy as np
from pedalboard import Pedalboard, load_plugin

PLUGIN_PATH = 'target/bundled/Colourizer Rs.vst3'


def run_case(sample_rate: int, channels: int, freq: float):
    plugin = load_plugin(PLUGIN_PATH)
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
        'sample_rate': sample_rate,
        'channels': channels,
        'frequency': freq,
        'shape': list(processed.shape),
        'min': float(processed.min()),
        'max': float(processed.max()),
        'elapsed_sec': elapsed,
    }


def main():
    results = []
    for sr in (44100, 48000, 96000):
        for ch in (2,):  # plugin requires stereo
            for freq in (220.0, 440.0, 880.0, 1760.0):
                results.append(run_case(sr, ch, freq))

    Path('test_results.md').write_text('# Python pedalboard test results\n\n')
    with open('test_results.md', 'a') as f:
        for r in results:
            f.write(
                f"- sr={r['sample_rate']} ch={r['channels']} f={r['frequency']}" \
                f" shape={r['shape']} min={r['min']:.3f} max={r['max']:.3f}" \
                f" elapsed={r['elapsed_sec']:.4f}s\n"
            )
    with open('test_results.json', 'w') as f:
        json.dump(results, f, indent=2)


if __name__ == '__main__':
    main()
