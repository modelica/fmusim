# fmusim

fmusim is a command line tool to work with Functions Mock-up Units, written in Rust.

## Highlights

- Supports FMI version 2.0 and 3.0.
- Supports Co-Simulation and Model Exchange.
- Builds platform binaries from source code FMUs using CMake.
- Logs FMI API calls.
- Reads and writes CSVs.
- Creates plots from simulation results.

See the [tutorial](tutorial.md) to get started.

## Installation

### Standalone installer

Install fmusim with our standalone installer:

=== "macOS and Linux"

    ```bash
    curl -LsSf https://raw.githubusercontent.com/modelica/fmusim/refs/heads/main/install.sh | sh
    ```

=== "Windows"

    ```powershell
    powershell -ExecutionPolicy ByPass -c "irm https://raw.githubusercontent.com/modelica/fmusim/refs/heads/main/install.ps1 | iex"
    ```

### GitHub Releases

fmusim release artifacts can be downloaded directly from [GitHub Releases](https://github.com/modelica/fmusim/releases).

Each release page includes binaries for all supported platforms.
