use std::{fs, vec};

use annotate_snippets::{AnnotationKind, Level, Renderer, Snippet, renderer::DecorStyle};
use anstream::eprintln;
use anstyle::Style;
use anyhow::Context;
use fmi_rs::{
    build_description::BuildDescription,
    dae::DaeManifest,
    model_description::{FMIMajorVersion, ValidationError},
    schema::{
        validate_build_description, validate_dae_manifest, validate_fmi2_model_description,
        validate_fmi3_model_description,
    },
    zip::get_zip_contents,
};
use std::io::IsTerminal;

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

fn render_xml_error(source: &str, err: &ValidationError) {
    let mut snippet = Snippet::source(source);

    if !err.range.is_empty() {
        snippet = snippet.path("modelDescription.xml");
    }

    for range in &err.range {
        snippet = snippet.annotation(AnnotationKind::Primary.span(range.clone()));
    }

    let report = &[Level::ERROR.primary_title(&err.message).element(snippet)];

    let mut renderer = if std::io::stderr().is_terminal() {
        Renderer::styled().decor_style(DecorStyle::Unicode)
    } else {
        Renderer::plain()
    };

    let term_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(120);

    dbg!(term_width);

    renderer = renderer.term_width(term_width);

    anstream::eprintln!("{}", renderer.render(report));
}

pub fn validate_fmu(fmu_file: &str) -> anyhow::Result<()> {
    let red = Style::new()
        .bold()
        .fg_color(Some(anstyle::AnsiColor::BrightRed.into()));
    let green = Style::new()
        .bold()
        .fg_color(Some(anstyle::AnsiColor::BrightGreen.into()));

    eprintln!("    {green}Validating ZIP archive{green:#}");

    let problems = validate_zip_archive(fmu_file);

    for problem in problems {
        eprintln!("{red}error{red:#}: {problem}");
    }

    let (unzipdir, xml_path, fmi_major_version) = prepare_fmu(fmu_file)?;

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

    let problems = match &fmi_major_version {
        FMIMajorVersion::V2 => fmi_rs::model_description::fmi2::ModelDescription::from_node(&root)
            .context("Failed to parse model description")?
            .validate(),
        FMIMajorVersion::V3 => fmi_rs::model_description::fmi3::ModelDescription::from_node(&root)
            .context("Failed to parse model description")?
            .validate(),
    };

    for problem in &problems {
        render_xml_error(&text, problem);
    }

    let build_description_path = unzipdir.path().join("sources/buildDescription.xml");

    if build_description_path.is_file() {
        eprintln!("    {green}Validating build description{green:#}");

        let document =
            fs::read(&build_description_path).context("Failed to read build description")?;

        let mut problems = validate_build_description(&document);

        if let Err(e) = BuildDescription::from_file(build_description_path) {
            problems.push(e.to_string());
        }

        for problem in problems {
            eprintln!("{red}error{red:#}: {problem}");
        }
    }

    let dae_manifest_path = unzipdir
        .path()
        .join("extra/org.fmi-standard.fmi-ls-dae/fmi-ls-manifest.xml");

    if dae_manifest_path.is_file() {
        eprintln!("    {green}Validating fmi-ls-dae manifest{green:#}");

        let document =
            fs::read(&dae_manifest_path).context("Failed to read fmi-ls-dae manifest")?;

        let mut problems = validate_dae_manifest(&document);

        if let Err(e) = DaeManifest::from_file(dae_manifest_path) {
            problems.push(e.to_string());
        }

        for problem in problems {
            eprintln!("{red}error{red:#}: {problem}");
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Validation failed"))
    }
}
