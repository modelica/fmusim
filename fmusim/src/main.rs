#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod build;
mod info;
mod simulate;
mod validate;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use colored::Colorize;
use crash_handler::{CrashEventResult, CrashHandler, make_crash_event};
use fmi_rs::{
    model_description::{FMIMajorVersion, peek_fmi_major_version},
    util::{extract_zip_archive, get_zip_contents},
};
use serde::Deserialize;
use std::{error::Error, io, path::PathBuf};
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

#[derive(Parser, Debug)]
#[command(
    name = "fmusim",
    version,
    propagate_version = true,
    about = "Simulate and validate Functional Mock-up Units"
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
    /// Generate shell completion scripts
    #[command(hide = true)]
    Completion {
        /// The shell to generate completions for
        #[arg(value_enum)]
        shell: ShellArg,
    },
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
    /// Validate an FMU
    Validate {
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
    /// Build the platform binary for an FMU with CMake
    Build(BuildArgs),
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

#[macro_export]
macro_rules! error {
    ($message:expr) => {
        use colored::Colorize;
        eprintln!("{}: {}", "error".red().bold(), $message);
        return ExitCode::FAILURE;
    };
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

    let result: Result<(), Box<dyn Error>> = match &cli.command {
        Commands::Completion { shell } => generate_completions(shell),
        Commands::Info { fmu_file } => info::show_fmu_info(fmu_file),
        Commands::List { fmu_file } => list_fmu_contents(fmu_file),
        Commands::Validate { fmu_file } => validate::validate_fmu(fmu_file),
        Commands::Simulate(args) => simulate::simulate_fmu(args),
        Commands::SimulateConfig { config_file } => simulate::simulate_config(config_file),
        Commands::Build(args) => build::build_platform_binary(args),
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}: {}", "error".bright_red().bold(), e);
            ExitCode::FAILURE
        }
    }
}

fn generate_completions(shell: &ShellArg) -> Result<(), Box<dyn Error + 'static>> {
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

fn list_fmu_contents(fmu_file: &str) -> Result<(), Box<dyn Error>> {
    get_zip_contents(fmu_file)
        .map_err(|e| format!("Failed to list FMU archive contents: {e}"))?
        .iter()
        .for_each(|entry| println!("{entry}"));
    Ok(())
}

/// Common logic to extract an FMU and the detect its FMI major version
fn prepare_fmu<P: AsRef<Path>>(
    fmu_path: P,
) -> Result<(TempDir, PathBuf, FMIMajorVersion), Box<dyn Error>> {
    let unzipdir =
        TempDir::new().map_err(|e| format!("Failed to create temporary directory: {e}"))?;

    extract_zip_archive(fmu_path, &unzipdir).map_err(|e| format!("Failed to extract FMU: {e}"))?;

    let xml_path = unzipdir.path().join("modelDescription.xml");

    let fmi_major_version = peek_fmi_major_version(&xml_path)
        .map_err(|e| format!("Failed to determine FMI version: {e}"))?;

    Ok((unzipdir, xml_path, fmi_major_version))
}
