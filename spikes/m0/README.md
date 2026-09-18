# Milestone 0 spikes

Throwaway experiments that retired the v2 architecture's risks. Results and
conclusions are in `docs/v2-plan.md` under "Milestone 0 results". Each Rust
folder is its own crate: `cd` in and `cargo run --release`.

- `webcodecs-probe.html`: open in a headed Edge window (same engine as
  WebView2) and read the console, or the page, for encoder, decoder,
  MediaRecorder and device support. Headless mode has no GPU and reports
  hardware encoders as unsupported, so do not use it for this.
- `xcap-wgc-and-hook/`: xcap's Windows.Graphics.Capture recorder (frame rate
  over 3 s) and a `WH_KEYBOARD_LL` hook fed synthetic F24 presses.
- `windows-capture/`: the `windows-capture` crate capturing the primary
  monitor for 5 s with the cursor hidden, straight into its Media
  Foundation H.264 encoder, printing frame statistics and writing
  `spike.mp4`.

Run the capture spikes with something moving on screen; WGC only delivers a
frame when the screen changes.
