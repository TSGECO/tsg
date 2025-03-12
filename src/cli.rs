mod dot;
mod fa;
mod gtf;
mod json;
mod path;
mod vcf;

pub use dot::*;
pub use fa::*;
pub use gtf::*;
pub use json::*;
pub use path::*;
pub use vcf::*;

use clap::Subcommand;
use clap::ValueHint;
use std::path::PathBuf;

/// Command line interface for the TSG tool
///
/// # Examples
///
/// ```no_run
/// # Parse a TSG file
/// tsg parse input.tsg
///
/// # Convert to DOT format
/// tsg dot input.tsg -o output.dot
///
/// # Generate FASTA with reference genome
/// tsg fa input.tsg -r reference.fa -o output.fa
///
/// # Find all paths through the graph
/// tsg traverse input.tsg -o paths.txt
/// ```
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

        /// Path to the reference genome file
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        reference_genome: PathBuf,

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

        /// Output file path for the paths
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },
}
