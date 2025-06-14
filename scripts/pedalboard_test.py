import numpy as np
from pedalboard import Pedalboard, VST3Plugin


def main():
    plugin_path = 'target/bundled/Colourizer Rs.vst3'
    plugin = VST3Plugin(plugin_path)

    board = Pedalboard([plugin])

    sr = 48000
    samples = np.random.randn(2, sr).astype(np.float32)

    processed = board(samples, sr)
    print('Processed audio shape:', processed.shape)


if __name__ == '__main__':
    main()
