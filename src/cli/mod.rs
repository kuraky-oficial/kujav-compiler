use crate::errors::{KujavError, KujavResult};

#[derive(Debug)]
pub struct Cli {
    pub command: Commands,
}

#[derive(Debug)]
pub enum Commands {
    New { project: String },
    Build,
    Run,
    Check,
    Install,
    Update,
    Publish,
    Clean,
}

impl Cli {
    pub fn parse() -> KujavResult<Self> {
        let mut args = std::env::args().skip(1);
        let command = match args.next().as_deref() {
            Some("new") => {
                let project = args
                    .next()
                    .ok_or_else(|| KujavError::io("usage: kujav new <project>"))?;
                Commands::New { project }
            }
            Some("build") => Commands::Build,
            Some("run") => Commands::Run,
            Some("check") => Commands::Check,
            Some("install") => Commands::Install,
            Some("update") => Commands::Update,
            Some("publish") => Commands::Publish,
            Some("clean") => Commands::Clean,
            _ => {
                return Err(KujavError::io(
                    "usage: kujav <new|build|run|check|install|update|publish|clean>",
                ));
            }
        };
        Ok(Self { command })
    }
}
