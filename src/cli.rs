mod dot;
mod fa;
mod gtf;
mod json;
mod merge;
mod path;
mod query;
mod split;
mod vcf;

pub use dot::*;
pub use fa::*;
pub use gtf::*;
pub use json::*;
pub use merge::*;
pub use path::*;
pub use query::*;
pub use split::*;
pub use vcf::*;

use clap::Subcommand;
use clap::ValueHint;
use std::path::PathBuf;

/// Command line interface for the TSG tool
#[derive(Subcommand)]
pub enum Commands {
    /// Parse a TSG file and validate its structure
    Parse {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,
    },

    /// Convert a TSG file to FASTA format
    Fa {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Output file path for the FASTA
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },

    /// Convert a TSG file to GTF format
    Gtf {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Output file path for the GTF
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },

    /// Convert a TSG file to VCF format
    Vcf {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Output file path for the VCF
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },

    /// Convert a TSG file to DOT format for graph visualization
    Dot {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Output DOT file path
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },

    /// Convert a TSG file to JSON format
    Json {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Format JSON with indentation for better readability
        #[arg(short, long, default_value = "false")]
        pretty: bool,

        /// Output file path for the JSON
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },

    /// Find and enumerate all valid paths through the graph
    Traverse {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,

        // Output the text representation of the paths
        #[arg(short, long, default_value = "false")]
        text_path: bool,

        /// Output file path for the paths, default is stdout
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },

    /// Merge multiple TSG files into a single TSG file
    Merge {
        /// Input TSG file paths
        #[arg(required = true, action=clap::ArgAction::Append, value_hint = ValueHint::FilePath)]
        inputs: Vec<PathBuf>,

        /// Output file path for the merged TSG
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },

    /// Split a TSG file into multiple TSG files
    Split {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Output directory for the split TSG files
        #[arg(short, long, value_hint = ValueHint::DirPath)]
        output: Option<PathBuf>,
    },

    /// Query specific graphs from a TSG file
    Query {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Graph IDs to query, can be separated by commas
        #[arg(short, long)]
        ids: String,

        /// File containing graph IDs to query (one per line)
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        ids_file: Option<PathBuf>,

        /// Output file path for the queried graphs
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },
}
