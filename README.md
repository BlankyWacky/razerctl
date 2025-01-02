# razerctl

A rust library which allows you to control your mouse with Razer Synapse

## Requirements

- Windows operating system
- Rust 1.56 or higher

## Safety

This crate uses `unsafe` code for Windows API interactions but provides safe abstractions for users. All unsafe operations are thoroughly documented and contained within the implementation.

## Features

- ✅ Currently supports mouse movement/clicking
- ✅ Supports sending keyboard inputs
- ❌ No support for keyboard dictionary yet, you'll have to experiment yourself for now

## Installation

In your root project folder, run the following to add razerctl to your dependencies.

```bash
cargo add razerctl
```

## Quick Start

```rust
use std::io::Error;

use razerctl::{init, mouse_move};

fn main() -> Result<(), Error> {
    // Initialize with default settings
    init()?;
    
    // Move mouse to relative coordinates (100, 100)
    mouse_move(100, 100)?;
    
    Ok(())
}
```


## Examples

Run the included examples:

```bash
# Mouse clicking demo
cargo run --example left_click

# Basic mouse movement demo
cargo run --example mouse_move1

# Fast mouse movement demo
cargo run --example mouse_move2
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.