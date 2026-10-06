use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anstream::println;
use anyhow::Context;
use askama::Template;
use fmi_rs::model_description::fmi2::{
    Causality as Causality2, ModelDescription as ModelDescription2,
};
use fmi_rs::model_description::fmi3::{
    Causality as Causality3, ModelDescription as ModelDescription3,
};
use fmi_rs::model_description::{FMIMajorVersion, peek_fmi_major_version};

use crate::simulate::calculate_simulation_steps;
use crate::{SimulateArgs, prepare_fmu};

#[derive(Template)]
#[template(path = "config.toml.askama")]
struct ConfigTemplate {
    filename: String,
    start_time: f64,
    stop_time: f64,
    tolerance: f64,
    output_interval: f64,
    simulate_args: SimulateArgs,
    config_dir: PathBuf,
    working_dir: PathBuf,
    descriptions: HashMap<String, String>,
}

impl ConfigTemplate {
    fn relative_path(&self, path: &Path) -> String {
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.working_dir.join(path)
        };
        pathdiff::diff_paths(&absolute_path, &self.config_dir)
            .unwrap_or(absolute_path)
            .to_string_lossy()
            .replace('\\', "/")
    }
}

fn render_simulation_config(
    fmu_file: &str,
    xml_path: &Path,
    config_file: Option<&Path>,
) -> anyhow::Result<String> {
    let fmi_major_version = peek_fmi_major_version(xml_path)
        .context("Failed to read FMI version from model description")?;
    let working_dir = std::env::current_dir().context("Failed to get current directory")?;
    let config_dir = config_file
        .and_then(Path::parent)
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let config_dir = if config_dir.is_absolute() {
        config_dir.to_path_buf()
    } else {
        working_dir.join(config_dir)
    };
    let filename = Path::new(fmu_file)
        .file_stem()
        .context("FMU path has no filename stem")?
        .to_string_lossy()
        .into_owned();

    let mut simulate_args = SimulateArgs {
        fmu_file: fmu_file.to_owned(),
        ..Default::default()
    };

    let (default_experiment, descriptions) = match fmi_major_version {
        FMIMajorVersion::V2 => {
            let model_description = ModelDescription2::from_path(xml_path)
                .context("Failed to parse model description")?;

            let mut descriptions = HashMap::new();

            for variable in &model_description.modelVariables {
                if let Some(start) = variable.variableType.start() {
                    simulate_args
                        .start_values
                        .push((variable.name.clone(), start.clone()));
                }
                if variable.causality == Causality2::Output {
                    simulate_args.output_variables.push(variable.name.clone());
                }
                if let Some(description) = variable.description.clone() {
                    descriptions.insert(variable.name.clone(), description);
                }
            }

            (model_description.defaultExperiment, descriptions)
        }
        FMIMajorVersion::V3 => {
            let model_description = ModelDescription3::from_path(xml_path)
                .context("Failed to parse model description")?;

            let mut descriptions = HashMap::new();

            for variable in &model_description.modelVariables {
                if matches!(variable.causality, Causality3::Parameter)
                    && let Some(start) = variable.variableType.start()
                {
                    let start_value = (variable.name.clone(), start.join(" "));
                    simulate_args.start_values.push(start_value);
                }
                if variable.causality == Causality3::Output {
                    simulate_args.output_variables.push(variable.name.clone());
                }
                if let Some(description) = variable.description.clone() {
                    descriptions.insert(variable.name.clone(), description);
                }
            }

            (model_description.defaultExperiment, descriptions)
        }
    };

    if let Some(default_experiment) = &default_experiment {
        simulate_args.start_time = default_experiment
            .startTime
            .as_deref()
            .and_then(|value| value.parse().ok());
        simulate_args.stop_time = default_experiment
            .stopTime
            .as_deref()
            .and_then(|value| value.parse().ok());
        simulate_args.tolerance = default_experiment
            .tolerance
            .as_deref()
            .and_then(|value| value.parse().ok());
        simulate_args.output_interval = default_experiment
            .stepSize
            .as_deref()
            .and_then(|value| value.parse().ok());
    };

    let (start_time, stop_time, tolerance, output_interval) =
        calculate_simulation_steps(&simulate_args, default_experiment.as_ref(), None);

    let template = ConfigTemplate {
        filename: filename.clone(),
        start_time,
        stop_time,
        tolerance,
        output_interval,
        simulate_args,
        config_dir: config_dir.clone(),
        working_dir: working_dir.clone(),
        descriptions,
    };

    let mut template = template;
    let fmu_path = PathBuf::from(&template.simulate_args.fmu_file);
    template.simulate_args.fmu_file = template.relative_path(&fmu_path);

    template
        .render()
        .context("Failed to render simulation configuration template")
}

pub fn generate_config(fmu_file: &str, config_file: Option<&str>) -> anyhow::Result<()> {
    let (_unzipdir, xml_path, _fmi_major_version) = prepare_fmu(fmu_file)?;

    let config = render_simulation_config(fmu_file, &xml_path, config_file.map(Path::new))
        .context("Failed to generate simulation configuration")?;

    if let Some(output_file) = config_file {
        let path = Path::new(output_file);
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory '{}'", parent.display()))?;
        }
        fs::write(path, config)
            .with_context(|| format!("Failed to write configuration file '{}'", output_file))?;
    } else {
        println!("{config}");
    }

    Ok(())
}
