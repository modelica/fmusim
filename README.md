# fmusim

Simulate and validate Functional Mock-up Units

## Installation

Download the latest build from [GitHub Actions](https://github.com/modelica/fmusim/actions/)

## Build with cargo

* [install Rust](https://rust-lang.org/tools/install/)
* clone the repository recursively `git clone --recursive https://github.com/modelica/fmusim.git`
* build fmusim `cargo build --release`

## Usage

```
$ ./fmusim
Simulate and validate Functional Mock-up Units

Usage: fmusim <COMMAND>

Commands:
  info      Display information about an FMU
  validate  Validate an FMU
  simulate  Simulate an FMU
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
