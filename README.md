# fmusim

Simulate and validate Functional Mock-up Units

## Installation

Install fmusim with our standalone installers

```bash
# on macOS and Linux
curl -LsSf https://raw.githubusercontent.com/modelica/fmusim/refs/heads/main/install.sh | sh
```

```bash
# on Windows
powershell -ExecutionPolicy ByPass -c "irm https://raw.githubusercontent.com/modelica/fmusim/refs/heads/main/install.ps1 | iex"
```

or download the [latest release](https://github.com/modelica/fmusim/releases/latest/).

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
  list      List the files contained in an FMU archive
  pack      Create an FMU archive from a folder
  unpack    Unpack an FMU archive to a folder
  validate  Validate an FMU
  simulate  Simulate an FMU
  build     Build the platform binary for an FMU with CMake
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
