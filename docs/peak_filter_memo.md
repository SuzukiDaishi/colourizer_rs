# Peak filter principles

The original filter bank used a peaking EQ with 0 dB gain which acts as an
all‑pass filter. By subtracting the dry input from a high‑gain peaking filter we
obtain a narrow band‐pass response. A high `Q` makes the peak very sharp while a
large positive gain (40 dB) increases the difference between the centre
frequency and its neighbours.

Processing a sample `x[n]` with a peaking filter `H(z)` and subtracting the
input yields:

```
y[n] = H(z) * x[n] - x[n]
```

This emphasises the selected frequency while cancelling the rest of the signal.
Summing the results for enabled notes and subtracting the summed dry signal
isolates only the desired tones.
