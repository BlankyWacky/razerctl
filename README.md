# razerctl

A Rust port of the [original rzctl project](https://github.com/Sadmeme/rzctl), allows you to control your mouse with Razer Synapse

## Requirements

- Windows operating system
- Rust 1.56 or higher

## Safety

This crate uses `unsafe` code for Windows API interactions but provides safe abstractions for users. All unsafe operations are thoroughly documented and contained within the implementation.

## Features

- ✅ Currently supports mouse movement/clicking
- ❌ No support for keyboard yet

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
razerctl = "0.1.0"
```

## Quick Start

```rust
use std::io::Error;

use razerctl::{init, mouse_move};

fn main() -> Result<(), Error> {
    // Initialize with default settings
    init()?;
    
    // Move mouse to coordinates (100, 100)
    mouse_move(100, 100, true)?;
    
    Ok(())
}
```


## Examples

Run the included examples:

```bash
# Basic mouse movement demo
cargo run --example basic_mouse_move

# Fast mouse movement demo
cargo run --example fast_mouse_move
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.