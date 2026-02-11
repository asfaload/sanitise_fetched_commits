use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Plain,
    Json,
}

#[derive(Parser, Debug)]
#[command(
    name = "git-verify-tool",
    version,
    about = "Validate git commits against configurable rules"
)]
pub struct Cli {
    /// Path to the git repository
    #[arg(default_value = ".")]
    pub repo: String,

    /// Path to the validation rules JSON config
    #[arg(default_value = "validation_rules.json")]
    pub config: String,

    /// Show passes and warnings, not just violations
    #[arg(short, long)]
    pub verbose: bool,

    /// Report violations but always exit 0
    #[arg(long)]
    pub dry_run: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Plain)]
    pub format: OutputFormat,

    /// Git remote name to compare against
    #[arg(long, default_value = "origin")]
    pub remote: String,
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub verbose: bool,
    pub dry_run: bool,
    pub format: OutputFormat,
    pub remote: String,
}

impl From<&Cli> for RunOptions {
    fn from(cli: &Cli) -> Self {
        Self {
            verbose: cli.verbose,
            dry_run: cli.dry_run,
            format: cli.format.clone(),
            remote: cli.remote.clone(),
        }
    }
}
