# ABF Analytics

A desktop application for analyzing ABF (Axon Binary Format) electrophysiology data files. Built with Rust and the [Iced](https://github.com/iced-rs/iced) GUI framework.

## Features

- **Raw Signal Visualization** — View multi-channel voltage/current recordings with pan and zoom controls
- **Peak Detection** — Detect and highlight signal peaks with adjustable sensitivity
- **Fourier Transform (FFT) Analysis** — Frequency-domain analysis with optional smoothing and peak-only mode
- **Custom Sine Wave Overlay** — Overlay user-defined sine waves (frequency, amplitude, phase) on the FFT view for comparison
- **Signal Reconstruction** — Reconstruct signals from selected sine wave components with an adjustable vertical offset
- **Channel Selection** — Toggle individual channels on/off during viewing and analysis

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) 1.85 or later (edition 2024 used by the `abf_reader` sub-crate)
- A GPU with Vulkan, Metal, or DX12 support (required by the `wgpu` backend of Iced)

## Building

```bash
git clone https://github.com/Ciwiruk/abf_analytics.git
cd abf_analytics
cargo build --release
```

The compiled binary will be placed in `target/release/abf_analytics`.

## Usage

Run the application:

```bash
cargo run --release
```

1. On the start screen, type the path to your `.abf` file (or use the **Open File** button) and click **Load**.
2. Select which channels to display and click **Confirm**.
3. Use the **View** screen to browse the raw signal; pan left/right, adjust the time window, and change the graph height.
4. Switch to the **Analytics** screen to run peak detection or FFT analysis on the selected channels.

## Project Structure

```
abf_analytics/
├── src/
│   └── main.rs          # GUI application (Iced)
├── abf_reader/
│   └── src/lib.rs       # ABF v1 file format parser
├── Cargo.toml           # Workspace / application manifest
└── Cargo.lock
```

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
