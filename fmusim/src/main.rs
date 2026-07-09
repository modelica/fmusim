#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod build;
mod info;
mod simulate;
mod validate;

use anstream::{eprintln, println};
use anstyle::Style;
use anyhow::{self, Context};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use crash_handler::{CrashEventResult, CrashHandler, make_crash_event};
use fmi_rs::{
    model_description::{FMIMajorVersion, peek_fmi_major_version},
    zip::{create_zip_archive, extract_zip_archive, get_zip_contents},
};
use serde::Deserialize;
use std::{io, path::PathBuf};
use std::{path::Path, process::ExitCode};
use tempfile::TempDir;

#[derive(ValueEnum, Clone, Debug, Deserialize)]
enum InterfaceType {
    /// Model Exchange
    #[value(name = "me")]
    ModelExchange,
    /// Co-Simulation
    #[value(name = "cs")]
    CoSimulation,
}

#[derive(ValueEnum, Clone, Debug, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum SolverType {
    #[value(name = "euler")]
    Euler,
    #[default]
    #[value(name = "cvode")]
    Cvode,
}

fn parse_start_value(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();

    if parts.len() != 2 {
        return Err(format!(
            "Invalid format {s:?}. Expected \"variable_name=value\"."
        ));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .header(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::BrightGreen))),
        )
        .usage(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::BrightGreen))),
        )
        .literal(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::BrightCyan))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan))),
        )
}

#[derive(Parser, Debug)]
#[command(
    name = "fmusim",
    version,
    propagate_version = true,
    about = "Work with Functional Mock-up Units",
    styles = get_styles(),
)]
struct Cli {
    /// Path to the FMU file
    #[command(subcommand)]
    command: Commands,
}

// We map this to clap_complete's Shell enum
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellArg {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Display information about an FMU
    Info {
        /// Path to the FMU file
        fmu_file: String,
    },
    /// List the files contained in an FMU archive
    List {
        /// Path to the FMU file
        fmu_file: String,
    },
    /// Simulate an FMU
    Simulate(SimulateArgs),
    /// Simulate an FMU using a configuration file
    #[command(
        long_about = "Load simulation configuration from a TOML file with the same parameters as the simulate command.\n\n\
        Example configuration file:\n\n\
        fmu_file = \"BouncingBall.fmu\"\n\
        start_values = [[\"h\", \"1.5\"]]\n\
        stop_time = 3.0\n",
        hide = true
    )]
    SimulateConfig {
        /// Path to the configuration file
        config_file: String,
    },
    /// Validate an FMU
    Validate {
        /// Path to the FMU file
        fmu_file: String,
    },
    /// Build the platform binary for an FMU with CMake
    Build(BuildArgs),
    /// Create an FMU archive from a folder
    #[command(hide = true)]
    Pack {
        /// Path to the unpacked FMU
        source_dir: String,
        /// Path to the FMU file to create
        fmu_file: String,
    },
    /// Unpack an FMU archive to a folder
    #[command(hide = true)]
    Unpack {
        /// Path to the FMU file to unpack
        fmu_file: String,
        /// Path to the directory to unpack the FMU to
        target_dir: String,
    },
    /// Generate shell completion scripts
    #[command(hide = true)]
    Completion {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: ShellArg,
    },
}

#[derive(Debug, Args, Deserialize, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct SimulateArgs {
    /// Path to the FMU file
    fmu_file: String,

    /// Enable logging of FMI function calls
    #[arg(long)]
    log_fmi_calls: bool,

    /// File to write the log to (default: stderr)
    #[arg(long)]
    log_file: Option<String>,

    /// Interval for sampling the output variables
    #[arg(long)]
    output_interval: Option<f64>,

    /// Start time for the simulation
    #[arg(long)]
    start_time: Option<f64>,

    /// Stop time for the simulation
    #[arg(long)]
    stop_time: Option<f64>,

    /// Set stop time explicitly
    #[arg(long)]
    set_stop_time: bool,

    /// Relative tolerance for the simulation
    #[arg(long)]
    tolerance: Option<f64>,

    /// Set tolerance explicitly
    #[arg(long)]
    set_tolerance: bool,

    /// CSV file to read the input from
    #[arg(long)]
    input_file: Option<String>,

    /// CSV file to store the output
    #[arg(long)]
    output_file: Option<String>,

    /// Plot of up to 8 output variables
    #[arg(long)]
    show_plot: bool,

    /// Show markers in the plot
    #[arg(long)]
    show_markers: bool,

    /// Show events in the plot
    #[arg(long)]
    show_events: bool,

    /// CSV file to read the reference output from
    #[arg(long)]
    reference_file: Option<String>,

    /// Set start values for variables (format: variable_name=value)
    #[arg(long = "start-value", value_parser = parse_start_value)]
    start_values: Vec<(String, String)>,

    /// Record a specific variable
    #[arg(long)]
    output_variable: Vec<String>,

    /// Allow early return
    #[arg(long)]
    early_return_allowed: bool,

    /// Use event mode
    #[arg(long)]
    event_mode_used: bool,

    /// Enable FMU looging
    #[arg(long)]
    logging_on: bool,

    /// Show statisics
    #[arg(long)]
    show_stats: bool,

    /// The interface type to use
    #[arg(long, value_enum)]
    interface_type: Option<InterfaceType>,

    /// Step size for fixed step solver (default: output interval)
    #[arg(long)]
    fixed_step_size: Option<f64>,

    /// The solver to integrate Model Exchange FMUs
    #[arg(long, value_enum, default_value_t = SolverType::Cvode)]
    solver: SolverType,

    /// File to read the serialized FMU state from
    #[arg(long)]
    initial_fmu_state_file: Option<String>,

    /// File to write the serialized FMU state to
    #[arg(long)]
    final_fmu_state_file: Option<String>,
}

#[derive(Debug, Args)]
struct BuildArgs {
    /// Path to the FMU file
    fmu_file: String,

    /// Path to the generated CMake project directory (default: temporary directory)
    #[arg(long)]
    project_path: Option<String>,

    /// CMake generator to use (default: platform default)
    #[arg(long)]
    cmake_generator: Option<String>,

    /// CMake architecture to use (default: platform default)
    #[arg(long)]
    cmake_architecture: Option<String>,

    /// Additional CMake options to pass to the build system
    #[arg(long, value_delimiter = ' ')]
    cmake_options: Vec<String>,

    /// Create a debug build (default: release build)
    #[arg(long)]
    debug: bool,
}

#[derive(Debug, Args)]
struct TestArgs {
    /// Path to the FMU file
    fmu_file: String,

    /// Enable logging of FMI function calls
    #[arg(long)]
    log_fmi_calls: bool,
}

fn main() -> ExitCode {
    let _crash_handler = CrashHandler::attach(unsafe {
        make_crash_event(|_| {
            eprintln!("The application crashed due to an unexpected exception.");
            CrashEventResult::Handled(true)
        })
    })
    .expect("Failed to attach crash handler");

    let cli = Cli::parse();

    let result: anyhow::Result<()> = match &cli.command {
        Commands::Completion { shell } => generate_completions(shell),
        Commands::Info { fmu_file } => info::show_fmu_info(fmu_file),
        Commands::List { fmu_file } => list_fmu_contents(fmu_file),
        Commands::Pack {
            source_dir,
            fmu_file,
        } => create_zip_archive(source_dir, fmu_file)
            .with_context(|| format!("Failed to create FMU archive {fmu_file} from {source_dir}")),
        Commands::Unpack {
            fmu_file,
            target_dir,
        } => extract_zip_archive(fmu_file, target_dir)
            .with_context(|| format!("Failed to extract FMU archive {fmu_file} to {target_dir}")),
        Commands::Validate { fmu_file } => validate::validate_fmu(fmu_file),
        Commands::Simulate(args) => simulate::simulate_fmu(args),
        Commands::SimulateConfig { config_file } => simulate::simulate_config(config_file),
        Commands::Build(args) => build::build_platform_binary(args),
    };

    let red = Style::new()
        .bold()
        .fg_color(Some(anstyle::AnsiColor::BrightRed.into()));

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{red}error{red:#}: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn generate_completions(shell: &ShellArg) -> anyhow::Result<()> {
    let target_shell = match shell {
        ShellArg::Bash => Shell::Bash,
        ShellArg::Zsh => Shell::Zsh,
        ShellArg::Fish => Shell::Fish,
        ShellArg::PowerShell => Shell::PowerShell,
        ShellArg::Elvish => Shell::Elvish,
    };
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    clap_complete::generate(target_shell, &mut cmd, bin_name, &mut io::stdout());
    Ok(())
}

fn list_fmu_contents(fmu_file: &str) -> anyhow::Result<()> {
    get_zip_contents(fmu_file)
        .context("Failed to list FMU archive contents")?
        .iter()
        .for_each(|entry| println!("{entry}"));
    Ok(())
}

/// Common logic to extract an FMU and the detect its FMI major version
fn prepare_fmu<P: AsRef<Path>>(fmu_path: P) -> anyhow::Result<(TempDir, PathBuf, FMIMajorVersion)> {
    let unzipdir = TempDir::new().context("Failed to create temporary directory")?;

    extract_zip_archive(fmu_path, &unzipdir).context("Failed to extract FMU")?;

    let xml_path = unzipdir.path().join("modelDescription.xml");

    let fmi_major_version =
        peek_fmi_major_version(&xml_path).context("Failed to determine FMI version")?;

    Ok((unzipdir, xml_path, fmi_major_version))
}
