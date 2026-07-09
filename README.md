# fmusim

fmusim is a command line tool to work with Functions Mock-up Units, written in Rust.

## Highlights

- Supports FMI version 2.0 and 3.0.
- Supports Co-Simulation and Model Exchange.
- Builds platform binaries from source code FMUs using CMake.
- Logs FMI API calls.
- Reads and writes CSVs.
- Creates plots from simulation results. 

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

To try out fmusim you can download the [latest release](https://github.com/modelica/Reference-FMUs/releases/latest) of the Reference FMUs.

## Getting help

### Help menus

The `--help` flag can be used to view the help menu for a command, e.g., for `fmusim`:

```console
$ fmusim help
```

To view the help menu for a specific sub command, e.g., for `fmusim simulate`:

```console
$ fmusim help simulate
```

### Viewing the version

When seeking help, it's important to determine the version of fmusim that you're using — sometimes the problem is already solved in a newer version.

To check the installed version:

```console
$ fmusim --version
```

### Open an issue on GitHub

The [issue tracker](https://github.com/modelica/fmusim/issues) on GitHub is a good place to report bugs and request features.
Make sure to search for similar issues first, as it is common for someone else to encounter the same problem.

## Display information about an FMU

```console
$ fmusim info BouncingBall.fmu
Model Information

FMI Version:       3.0
Model Name:        BouncingBall
Platforms:         c-code, aarch64-darwin, aarch64-linux, x86-windows, x86_64-darwin, x86_64-linux, x86_64-windows
Continuous States: 2
Event Indicators:  1
Model Variables:   8
Generation Date:   2026-07-09T12:21:24.670948+00:00
Generation Tool:   Reference FMUs (7b3fc08)
Description:       This model calculates the trajectory, over time, of a ball dropped from a height of 1 m

Model Variables

Name   | Description
-------|---------------------------------------------
time   | Simulation time
h      | Position of the ball
der(h) | Derivative of h
v      | Velocity of the ball
der(v) | Derivative of v
g      | Gravity acting on the ball
e      | Coefficient of restitution
v_min  | Velocity below which the ball stops bouncing
```

## List the contents of an FMU

```console
$ fmusim list BouncingBall.fmu
binaries/
binaries/x86_64-linux/
binaries/x86_64-linux/BouncingBall.so
binaries/x86_64-windows/
binaries/x86_64-windows/BouncingBall.dll
documentation/
documentation/index.html
documentation/result.svg
modelDescription.xml
...
```

## Pack and unpack an FMU

Unpack the contents of an FMU archive to a directory

```console
$ fmusim unpack BouncingBall.fmu BouncingBall
```

Create an FMU archive from a directory

```console
$ fmusim pack BouncingBall BouncingBall.fmu
```

## Validate an FMU

```console
$ fmusim validate BouncingBall.fmu
    Validating ZIP archive
    Validating model description
```

## Simulate an FMU

Simulate an FMU with the default settings and plot the outputs

```console
$ fmusim simulate BoucingBall.fmu --show-plot
```

![BoucingBall](docs/plot.svg)

See [simulate](simluate.md) to learn more.

## Build the platform binary for an FMU

```console
fmusim build BouncingBall.fmu  
Creating CMake project
Configuring CMake project
...
Building CMake project
...
Finished
```

## License

fmusim is released under the BSD-2-Clause license.
