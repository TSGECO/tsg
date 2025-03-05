mod dot;
mod fa;
mod gtf;
mod json;
mod vcf;

pub use dot::*;
pub use fa::*;
pub use gtf::*;
pub use json::*;
pub use vcf::*;

use clap::Subcommand;
use clap::ValueHint;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum Commands {
    /// Parse a TSG file and validate its structure
    Parse {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,
    },

    /// convert TSG file to fa file
    Fa {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Reference genome path
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        reference_genome: PathBuf,

        /// Output file path for the FASTA
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },

    /// convert TSG file to gtf file
    Gtf {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Output file path for the GTF
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },

    /// convert TSG to vcf file
    Vcf {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Output file path for the VCF
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },

    /// Convert a TSG file to DOT format for visualization
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

        /// If output pretty json
        #[arg(short, long, default_value = "false")]
        pretty: bool,

        /// Output file path for the JSON
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },

    /// Find all valid paths through the graph
    Traverse {
        /// Input TSG file path
        #[arg(required = true, value_hint = ValueHint::FilePath)]
        input: PathBuf,

        /// Output file path for the paths
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        output: Option<PathBuf>,
    },
}
