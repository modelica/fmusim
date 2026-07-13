use crate::prepare_fmu;
use anstream::println;
use anstyle::Style;
use fmi_rs::model_description::FMIMajorVersion;
use std::io::{self, IsTerminal};

struct GridStyle {
    horizontal: &'static str,
    cross: &'static str,
    vertical: &'static str,
}

impl GridStyle {
    fn detect() -> Self {
        // Check if stdout is connected to a real terminal
        if io::stdout().is_terminal() {
            // Fancy Unicode for terminal display
            Self {
                horizontal: "─",
                cross: "┼",
                vertical: "│",
            }
        } else {
            // Safe ASCII fallback for files / pipes / redirects
            Self {
                horizontal: "-",
                cross: "+",
                vertical: "|",
            }
        }
    }
}

pub fn show_fmu_info(fmu_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let grid = GridStyle::detect();

    let (unzipdir, xml_path, fmi_major_version) = prepare_fmu(fmu_file)?;

    let entries = std::fs::read_dir(unzipdir.path().join("binaries"))?;

    let mut platform_dirs: Vec<String> = vec![];

    if unzipdir.path().join("sources").is_dir() {
        platform_dirs.push("c-code".to_string());
    }

    platform_dirs.extend(
        entries
            .filter_map(|res| res.ok())
            .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
            .filter_map(|entry| entry.file_name().into_string().ok()),
    );

    let bold = Style::new().bold();

    match fmi_major_version {
        FMIMajorVersion::V2 => {
            let model_description =
                fmi_rs::model_description::fmi2::ModelDescription::from_path(&xml_path)
                    .map_err(|e| format!("Failed to read model description: {e}"))?;

            let mut interface_types = vec![];

            if model_description.modelExchange.is_some() {
                interface_types.push("Model Exchange");
            }

            if model_description.coSimulation.is_some() {
                interface_types.push("Co-Simulation");
            }

            println!("{bold}Model Information{bold:#}");
            println!();
            println!("FMI Version:       2.0");
            println!("Interface Types:   {}", interface_types.join(", "));
            println!("Model Name:        {}", model_description.modelName);
            println!("Platforms:         {}", platform_dirs.join(", "));
            println!("Continuous States: {}", model_description.derivatives.len());
            println!(
                "Event Indicators:  {}",
                model_description.numberOfEventIndicators
            );
            println!(
                "Model Variables:   {}",
                model_description.modelVariables.len()
            );
            println!(
                "Generation Date:   {}",
                model_description.generationDateAndTime.unwrap_or_default()
            );
            println!(
                "Generation Tool:   {}",
                model_description.generationTool.unwrap_or_default()
            );
            println!(
                "Description:       {}",
                model_description.description.unwrap_or_default()
            );
            println!();
            println!("{bold}Model Variables{bold:#}");
            println!();

            let terminal_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(120);

            let name_width = model_description
                .modelVariables
                .iter()
                .map(|v| v.name.len())
                .max() // Get the maximum length
                .unwrap_or(4); // Default to 4 if no variables or names are empty

            let description_width = terminal_width.saturating_sub(name_width + 3);

            let header = format!(
                "{bold}{:<nw$}{bold:#} {} {bold}{:<dw$}{bold:#}",
                "Name",
                grid.vertical,
                "Description",
                nw = name_width,
                dw = description_width
            );
            println!("{}", header);

            println!(
                "{}{}{}",
                grid.horizontal.repeat(name_width + 1),
                grid.cross,
                grid.horizontal.repeat(description_width + 1)
            );

            for variable in &model_description.modelVariables {
                println!(
                    "{:<nw$} │ {:<dw$}",
                    variable.name,
                    variable.description.as_deref().unwrap_or_default(),
                    nw = name_width,
                    dw = description_width
                );
            }
        }
        FMIMajorVersion::V3 => {
            let model_description =
                fmi_rs::model_description::fmi3::ModelDescription::from_path(&xml_path)
                    .map_err(|e| format!("Failed to read model description: {e}"))?;

            let mut interface_types = vec![];

            if model_description.modelExchange.is_some() {
                interface_types.push("Model Exchange");
            }

            if model_description.coSimulation.is_some() {
                interface_types.push("Co-Simulation");
            }

            if model_description.scheduledExecution.is_some() {
                interface_types.push("Scheduled Execution");
            }

            println!("{bold}Model Information{bold:#}");
            println!();
            println!("FMI Version:       {}", model_description.fmiVersion);
            println!("Interface Types:   {}", interface_types.join(", "));
            println!("Model Name:        {}", model_description.modelName);
            println!("Platforms:         {}", platform_dirs.join(", "));
            println!("Continuous States: {}", model_description.derivatives.len());
            println!(
                "Event Indicators:  {}",
                model_description.eventIndicators.len()
            );
            println!(
                "Model Variables:   {}",
                model_description.modelVariables.len()
            );
            println!(
                "Generation Date:   {}",
                model_description.generationDateAndTime.unwrap_or_default()
            );
            println!(
                "Generation Tool:   {}",
                model_description.generationTool.unwrap_or_default()
            );
            println!(
                "Description:       {}",
                model_description.description.unwrap_or_default()
            );
            println!();
            println!("{bold}Model Variables{bold:#}");
            println!();

            let terminal_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(120);

            let name_width = model_description
                .modelVariables
                .iter()
                .map(|v| v.name.len())
                .max() // Get the maximum length
                .unwrap_or(4); // Default to 4 if no variables or names are empty

            let description_width = terminal_width.saturating_sub(name_width + 3);

            let header = format!(
                "{bold}{:<nw$}{bold:#} {} {bold}{:<dw$}{bold:#}",
                "Name",
                grid.vertical,
                "Description",
                nw = name_width,
                dw = description_width
            );
            println!("{}", header);

            println!(
                "{}{}{}",
                grid.horizontal.repeat(name_width + 1),
                grid.cross,
                grid.horizontal.repeat(description_width + 1)
            );

            for variable in &model_description.modelVariables {
                println!(
                    "{:<nw$} {} {:<dw$}",
                    variable.name,
                    grid.vertical,
                    variable.description.as_deref().unwrap_or_default(),
                    nw = name_width,
                    dw = description_width
                );
            }
        }
    }

    Ok(())
}
