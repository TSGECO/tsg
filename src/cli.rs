mod dot;
mod fa;

pub use dot::*;
pub use fa::*;

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

    /// convert tsg format to fa format
    Fa {
        /// Input TSG file path
        #[arg(required = true)]
        input: PathBuf,

        /// Reference genome path
        #[arg(short, long)]
        reference_genome: PathBuf,

        /// Output file path for the FASTA
        #[arg(short, long)]
        output: Option<PathBuf>,
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
