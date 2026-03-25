use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "k8s-diff-explainer",
    about = "Explain rollout risk of Kubernetes manifest changes"
)]
pub struct Cli {
    /// Old/baseline manifest file
    pub old: PathBuf,
    /// New/target manifest file
    pub new: PathBuf,
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
    /// Write report to file (stdout still used for summary in text mode)
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Only print a one-line summary to stdout (full report can still go to --output)
    #[arg(long)]
    pub summary_only: bool,
    /// Exit non-zero if any workloads are removed in the new manifest
    #[arg(long)]
    pub fail_on_removals: bool,
    /// Fail CI when overall risk meets or exceeds this level
    #[arg(long = "fail-on", value_enum)]
    pub fail_on: Option<FailThreshold>,
    /// Enable experimental rules (selector drift, etc.)
    #[arg(long)]
    pub experimental: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum FailThreshold {
    Medium,
    High,
}
