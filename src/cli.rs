mod dot;
mod fa;
mod gtf;
mod vcf;

pub use dot::*;
pub use fa::*;
pub use gtf::*;
pub use vcf::*;

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

    /// convert tsg format to gtf format
    Gtf {
        /// Input TSG file path
        #[arg(required = true)]
        input: PathBuf,

        /// Output file path for the GTF
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// convert tsg format to vcf format
    Vcf {
        /// Input TSG file path
        #[arg(required = true)]
        input: PathBuf,

        /// Output file path for the VCF
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
