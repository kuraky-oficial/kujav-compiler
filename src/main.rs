mod cli;
mod compiler;
mod core;
mod errors;
mod package;
mod parser;
mod reader;
mod toml_config;

use std::fs;
use std::path::Path;

use cli::{Cli, Commands};
use errors::{KujavError, KujavResult};
use package::lockfile::write_lockfile;
use package::resolver::validate_java_classpath;
use toml_config::KujavToml;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> KujavResult<()> {
    let cli = Cli::parse()?;
    match cli.command {
        Commands::New { project } => new_project(&project),
        Commands::Build => build_project(),
        Commands::Run => run_project(),
        Commands::Check => check_project(),
        Commands::Install => {
            println!("kujav install: dependency fetch pipeline pending registry integration");
            Ok(())
        }
        Commands::Update => {
            println!("kujav update: semver resolver upgrade pipeline pending");
            Ok(())
        }
        Commands::Publish => {
            println!("kujav publish: registry client pipeline pending authentication");
            Ok(())
        }
        Commands::Clean => clean_project(),
    }
}

fn new_project(project: &str) -> KujavResult<()> {
    let root = Path::new(project);
    fs::create_dir_all(root.join("src"))?;
    let toml = format!(
        "[package]\nname = \"{project}\"\nversion = \"0.1.0\"\nmain = \"src/main.kj\"\nedition = \"2026\"\n\n[dependencies]\n\n[java]\nclasspath = []\n"
    );
    fs::write(root.join("kujav.toml"), toml)?;
    fs::write(
        root.join("src/main.kj"),
        "function main(): Int\n    local answer: Int = 42\n    print answer\n    return 0\nend\n",
    )?;
    println!("Created Kujav project: {project}");
    Ok(())
}

fn load_project() -> KujavResult<(KujavToml, String)> {
    let cfg = KujavToml::from_path("kujav.toml")?;
    let source = fs::read_to_string(&cfg.package.main)
        .map_err(|_| KujavError::io(format!("missing source file '{}'", cfg.package.main)))?;
    Ok((cfg, source))
}

fn check_project() -> KujavResult<()> {
    let (_cfg, source) = load_project()?;
    compiler::pipeline::check_only(&source)?;
    println!("check finished without errors");
    Ok(())
}

fn build_project() -> KujavResult<()> {
    let (cfg, source) = load_project()?;
    fs::create_dir_all("target")?;
    write_lockfile(&cfg)?;
    validate_java_classpath(&cfg)?;

    let class_path = format!("target/{}.class", cfg.package.name);
    compiler::pipeline::compile_to_class(&cfg.package.name, &source, &class_path)?;

    let jar_path = format!("target/{}.jar", cfg.package.name);
    compiler::pipeline::package_jar(&cfg, &class_path, &jar_path)?;
    println!("Built {}", jar_path);
    Ok(())
}

fn run_project() -> KujavResult<()> {
    let cfg = KujavToml::from_path("kujav.toml")?;
    let jar_path = format!("target/{}.jar", cfg.package.name);
    if !Path::new(&jar_path).exists() {
        build_project()?;
    }
    let status = std::process::Command::new("java")
        .arg("-jar")
        .arg(&jar_path)
        .status()
        .map_err(|_| KujavError::io("failed to execute java runtime"))?;
    if !status.success() {
        return Err(KujavError::bytecode(
            "java runtime returned non-zero status",
        ));
    }
    Ok(())
}

fn clean_project() -> KujavResult<()> {
    if Path::new("target").exists() {
        fs::remove_dir_all("target")?;
    }
    println!("cleaned target/");
    Ok(())
}
