use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "l10n-rust")]
#[command(about = "Rust localization generator")]
pub(crate) struct Cli {
    #[arg(long, default_value = "l10n-generator.config.yaml")]
    pub(crate) config: String,
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Diagnose(DiagnoseArgs),
    Oauth2Helper,
}

#[derive(Debug, Parser)]
pub(crate) struct DiagnoseArgs {
    #[arg(long, default_value = "test.config.yaml")]
    pub(crate) config: String,
}
