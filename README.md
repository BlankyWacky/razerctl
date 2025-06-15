# razerctl

[![crates.io](https://img.shields.io/crates/v/razerctl.svg)](https://crates.io/crates/razerctl)
[![Docs.rs](https://docs.rs/razerctl/badge.svg)](https://docs.rs/razerctl)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A low-level Rust library for controlling Razer mouse and keyboard input directly through the Razer Synapse driver on Windows.

## Features

- ✅ **Relative Mouse Movement**: Move the mouse cursor by a given x/y offset.
- ✅ **Mouse Button Clicks**: Simulate press and release events for all standard mouse buttons (Left, Right, Middle, X1, X2).
- ✅ **Keyboard Key Presses**: Simulate key down and key up events for a comprehensive set of keyboard keys.
- ✅ **Safe Abstractions**: Provides a safe and simple public API that handles the underlying `unsafe` Windows API and driver interactions.

## How It Works

This library works by locating the symbolic link to the `RZCONTROL` device created by the Razer Synapse driver and communicating with it directly using `DeviceIoControl`.

Keyboard input is not sent using standard Windows scan codes. Instead, this crate implements a custom translation layer that converts standard Virtual-Key (VK) codes into the specific `MakeCode` values the Razer driver expects. This logic was ported from the IbInputSimulator C++ project.

## Requirements

- Windows operating system
- Razer Synapse 3 installed
- Rust 1.56 or higher

## Installation

**1. Add `razerctl` to your project:**
```bash
cargo add razerctl
```

**2. (Recommended) Add `win_key_codes` for easy keyboard control:**

For sending keyboard input, you need to provide Windows Virtual-Key codes. The `win_key_codes` crate provides convenient constants for these codes.

```bash
cargo add win_key_codes
```

## Quick Start

Here is a quick example of how to initialize the library, move the mouse, and press the 'A' key.

```rust
use std::{io::Error, thread, time::Duration};
use razerctl::{init, mouse_move, key_down, key_up};
use win_key_codes::VK_A; // Use a constant for the 'A' key

fn main() -> Result<(), Error> {
    // 1. Initialize the connection to the Razer driver
    init()?;
    println!("Razer driver initialized.");

    // 2. Move the mouse relatively by (100, 100) pixels
    println!("Moving mouse...");
    mouse_move(100, 100)?;

    // Give a moment for the mouse move to be visible
    thread::sleep(Duration::from_secs(1));

    // 3. Simulate pressing and releasing the 'A' key
    println!("Pressing the 'A' key...");
    key_down(VK_A)?;
    thread::sleep(Duration::from_millis(50)); // Hold the key for 50ms
    key_up(VK_A)?;

    println!("Done!");
    Ok(())
}
```

## Safety

This crate uses `unsafe` code to interface with the Windows API and the device driver. However, the public API is designed to be a completely safe abstraction over these details. All `unsafe` blocks are contained internally and have been written with care.

## Examples

You can run the included examples from the project root:

```bash
# --- Mouse Examples ---
cargo run --example left_click
cargo run --example mouse_move1
cargo run --example mouse_move2

# --- Keyboard Example ---
cargo run --example keyboard_test
```

## Contributing

Contributions are welcome! If you find a bug or have an idea for an improvement, please feel free to submit an issue or a pull request.

## License

This project is licensed under the MIT License.