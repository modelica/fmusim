use std::path::{Path, PathBuf};

use crate::BuildArgs;
use anstream::{eprintln, println};
use anstyle::Style;
use anyhow::{self, Context};
use fmi_rs::util::create_cmake_project;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tempfile::TempDir;

pub fn build_platform_binary(args: &BuildArgs) -> anyhow::Result<()> {
    let green = Style::new()
        .bold()
        .fg_color(Some(anstyle::AnsiColor::BrightGreen.into()));

    let (project_path, _temp_dir) = if let Some(ref path) = args.project_path {
        (PathBuf::from(path), None)
    } else {
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        (temp_dir.path().to_path_buf(), Some(temp_dir))
    };

    eprintln!("{green}Creating CMake project{green:#}");

    create_cmake_project(Path::new(&args.fmu_file), project_path.as_path())?;

    eprintln!("{green}Configuring CMake project{green:#}");

    let build_path = project_path
        .join("build")
        .to_string_lossy()
        .replace('\\', "/");

    let source_path = project_path.to_string_lossy().replace('\\', "/");

    let mut command = Command::new("cmake");

    command
        .arg("-B")
        .arg(&build_path)
        .arg("-S")
        .arg(&source_path);

    if let Some(generator) = &args.cmake_generator {
        command.arg("-G").arg(generator);
    }

    if let Some(architecture) = &args.cmake_architecture {
        command.arg("-A").arg(architecture);
    }

    for option in &args.cmake_options {
        command.arg(format!("-D{}", option));
    }

    if !args.debug {
        command.arg("-DCMAKE_BUILD_TYPE=Release");
    } else {
        command.arg("-DCMAKE_BUILD_TYPE=Debug");
    }

    run_command_with_progress(&mut command)?;

    eprintln!("{green}Building CMake project{green:#}");

    let mut command = Command::new("cmake");

    command.arg("--build").arg(&build_path);

    if !args.debug {
        command.arg("--config").arg("Release");
    } else {
        command.arg("--config").arg("Debug");
    }

    run_command_with_progress(&mut command)?;

    eprintln!("{green}Finished{green:#}");

    Ok(())
}

/// Runs a command, streams its output to the terminal
fn run_command_with_progress(cmd: &mut Command) -> anyhow::Result<()> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to start {:?}", cmd.get_program()))?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            println!("{}", line?);
        }
    }

    match child.wait() {
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                anyhow::bail!("{:?} failed with status {}", cmd.get_program(), status)
            }
        }
        Err(e) => anyhow::bail!("{:?} failed: {}", cmd.get_program(), e),
    }
}
