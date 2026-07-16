use std::{fs, vec};

use anstream::eprintln;
use anstyle::Style;
use anyhow::Context;
use fmi_rs::{
    model_description::FMIMajorVersion,
    schema::{validate_fmi2_model_description, validate_fmi3_model_description},
    zip::get_zip_contents,
};

use crate::prepare_fmu;

/// Validates a ZIP archive
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

pub fn validate_fmu(fmu_file: &str) -> anyhow::Result<()> {
    let bold = Style::new().bold();
    let red = Style::new()
        .bold()
        .fg_color(Some(anstyle::AnsiColor::BrightRed.into()));
    let green = Style::new()
        .bold()
        .fg_color(Some(anstyle::AnsiColor::BrightGreen.into()));
    let arrow = Style::new()
        .bold()
        .fg_color(Some(anstyle::AnsiColor::Cyan.into()));

    eprintln!("    {green}Validating ZIP archive{green:#}");

    let problems = validate_zip_archive(fmu_file);

    for problem in problems {
        eprintln!("{red}error{red:#}: {problem}");
    }

    let (_unzipdir, xml_path, fmi_major_version) = prepare_fmu(fmu_file)?;

    eprintln!("    {green}Validating model description{green:#}");

    let document = fs::read(&xml_path)?;

    let problems = match fmi_major_version {
        FMIMajorVersion::V2 => validate_fmi2_model_description(&document),
        FMIMajorVersion::V3 => validate_fmi3_model_description(&document),
    };

    for problem in problems {
        eprintln!("{red}error{red:#}: {problem}");
    }

    let text = std::fs::read_to_string(xml_path).context("Failed to read model description")?;

    let opt = roxmltree::ParsingOptions {
        allow_dtd: true,
        ..roxmltree::ParsingOptions::default()
    };

    let doc = roxmltree::Document::parse_with_options(&text, opt)
        .context("Failed to parse model description")?;

    let root = doc.root_element();

    let mut problems = vec![];

    match &fmi_major_version {
        FMIMajorVersion::V2 => {
            let model_description =
                fmi_rs::model_description::fmi2::ModelDescription::from_node(&root)
                    .context("Failed to parse model description")?;
            problems.extend(model_description.validate());
        }
        FMIMajorVersion::V3 => {
            let model_description =
                fmi_rs::model_description::fmi3::ModelDescription::from_node(&root)
                    .context("Failed to parse model description")?;
            problems.extend(model_description.validate());
        }
    };

    let terminal_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(120);

    let max_width = terminal_width - 8;

    for problem in problems.iter() {
        eprintln!("{red}error{red:#}: {bold}{}{bold:#}", problem.message);

        if let Some(range) = problem.range.last() {
            let start_pos = doc.text_pos_at(range.start);
            eprintln!(
                "     {arrow}-->{arrow:#} modelDescription.xml:{}:{}",
                start_pos.row, start_pos.col
            );
        }

        for (j, range) in problem.range.iter().enumerate() {
            let start_pos = doc.text_pos_at(range.start);
            let end_pos = doc.text_pos_at(range.end);
            let start_line = (start_pos.row - 1) as usize;
            let end_line = (end_pos.row - 1) as usize;

            if j == 0 {
                eprintln!("      {arrow}|{arrow:#}");
            } else {
                eprintln!("  {arrow}...{arrow:#}");
            }

            for (i, line) in text.lines().enumerate() {
                if i >= start_line && i <= end_line {
                    let text = if line.len() > max_width {
                        let limit = max_width.saturating_sub(3);
                        format!("{line:.limit$}{arrow}...{arrow:#}")
                    } else {
                        line.to_string()
                    };

                    let prefix = if i == start_line {
                        format!("{arrow}{:>5}{arrow:#} {arrow}|{arrow:#} ", i + 1)
                    } else {
                        format!("      {arrow}|{arrow:#} ")
                    };
                    eprintln!("{}{}", prefix, text);
                }
            }
            eprintln!("      {arrow}|{arrow:#}");
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Validation failed"))
    }
}
