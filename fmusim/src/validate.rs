use std::vec;

use colored::Colorize;
use fmi_rs::{model_description::FMIMajorVersion, util::get_zip_contents};
use fmi_rs_libxml2::validate_model_description_against_xsd;

use crate::{ValidateArgs, prepare_fmu};

/// Validate ZIP archive
fn validate_zip_archive(fmu_file: &str) -> Vec<String> {
    let mut problems = vec![];

    if let Ok(contents) = get_zip_contents(fmu_file) {
        for entry in contents {
            if entry.starts_with(['.', '/']) {
                problems.push(format!(
                    "Path '{entry}' starts with a dot ('.') or slash ('/')"
                ));
            }
            if entry.contains(r"\") {
                problems.push(format!("Path '{entry}' contains a backslash ('\\')"));
            }
        }
    } else {
        problems.push(format!("Failed to read ZIP archive: {fmu_file}"));
    }

    problems
}

pub fn validate_fmu(args: &ValidateArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "    Validating ZIP archive".green().bold());

    for problem in validate_zip_archive(&args.fmu_file) {
        println!("{}: {}", "error".red().bold(), problem);
    }

    let (_unzipdir, xml_path, fmi_major_version) = prepare_fmu(&args.fmu_file)?;

    println!("{}", "    Validating model description".green().bold());

    let problems = validate_model_description_against_xsd(&xml_path, fmi_major_version as i32);

    for problem in problems.iter() {
        println!("{}: {}", "error".red().bold(), problem);
    }

    let text = std::fs::read_to_string(xml_path)
        .map_err(|e| format!("Failed to read modelDescription.xml: {e}"))?;

    let opt = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..roxmltree::ParsingOptions::default()
    };

    let doc = roxmltree::Document::parse_with_options(&text, opt)
        .map_err(|e| format!("Failed to parse modelDescription.xml: {e}"))?;

    let root = doc.root_element();

    let mut problems = vec![];

    match &fmi_major_version {
        FMIMajorVersion::V2 => {
            let model_description =
                fmi_rs::model_description::fmi2::ModelDescription::from_node(&root)
                    .map_err(|e| format!("Failed to parse modelDescription.xml: {e}"))?;
            problems.extend(model_description.validate());
        }
        FMIMajorVersion::V3 => {
            let model_description =
                fmi_rs::model_description::fmi3::ModelDescription::from_node(&root)
                    .map_err(|e| format!("Failed to parse modelDescription.xml: {e}"))?;
            problems.extend(model_description.validate());
        }
    };

    let terminal_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(120);

    let max_width = terminal_width - 8;

    for problem in problems.iter() {
        println!("{}: {}", "error".red().bold(), problem.message.bold());

        if let Some(range) = problem.range.last() {
            let start_pos = doc.text_pos_at(range.start);
            println!(
                "     {} modelDescription.xml:{}:{}",
                "-->".cyan().bold(),
                start_pos.row,
                start_pos.col
            );
        }

        for (j, range) in problem.range.iter().enumerate() {
            let start_pos = doc.text_pos_at(range.start);
            let end_pos = doc.text_pos_at(range.end);
            let start_line = (start_pos.row - 1) as usize;
            let end_line = (end_pos.row - 1) as usize;

            if j == 0 {
                println!("      {}", "|".cyan().bold());
            } else {
                println!("  {}", "...".cyan().bold());
            }

            for (i, line) in text.lines().enumerate() {
                if i >= start_line && i <= end_line {
                    let text = if line.len() > max_width {
                        let limit = max_width.saturating_sub(3);
                        format!("{line:.limit$}{}", "...".cyan().bold())
                    } else {
                        line.to_string()
                    };

                    let prefix = if i == start_line {
                        format!(
                            "{:>5} {} ",
                            (i + 1).to_string().cyan().bold(),
                            "|".cyan().bold()
                        )
                    } else {
                        format!("      {} ", "|".cyan().bold())
                    };
                    println!("{}{}", prefix, text);
                }
            }
            println!("      {}", "|".cyan().bold());
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err("Validation failed".into())
    }
}
