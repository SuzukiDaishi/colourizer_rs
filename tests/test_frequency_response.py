import numpy as np
from scipy.signal import lfilter

SR = 44100
Q = 100.0
GAIN_DB = 20.0
A = 10 ** (GAIN_DB / 40)


def peak_coeff(freq: float):
    w0 = 2 * np.pi * freq / SR
    alpha = np.sin(w0) / (2 * Q)
    b0 = 1 + alpha * A
    b1 = -2 * np.cos(w0)
    b2 = 1 - alpha * A
    a0 = 1 + alpha / A
    a1 = -2 * np.cos(w0)
    a2 = 1 - alpha / A
    b = np.array([b0 / a0, b1 / a0, b2 / a0])
    a = np.array([1.0, a1 / a0, a2 / a0])
    return b, a


def band_energy(freq: float, tone: float) -> float:
    n = np.arange(SR)
    sine = np.sin(2 * np.pi * tone * n / SR)
    b, a = peak_coeff(freq)
    y = lfilter(b, a, sine)
    return np.mean(np.abs(y - sine))


def test_peak_filter_selectivity():
    on = band_energy(440.0, 440.0)
    off = band_energy(440.0, 450.0)
    assert on > 10 * off

