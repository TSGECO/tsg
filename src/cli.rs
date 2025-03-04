mod dot;

pub use dot::*;

use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum Commands {
    /// Parse a TSG file and validate its structure
    Parse {
        /// Input TSG file path
        #[arg(required = true)]
        input: PathBuf,
    },

    /// Convert a TSG file to DOT format for visualization
    Dot {
        /// Input TSG file path
        #[arg(required = true)]
        input: PathBuf,

        /// Output DOT file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Find all valid paths through the graph
    Traverse {
        /// Input TSG file path
        #[arg(required = true)]
        input: PathBuf,

        /// Output file path for the paths
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
