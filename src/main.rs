mod graph;
mod io;

use tracing::info;
use tracing_subscriber;

use anyhow::Result;
use clap::{Parser, Subcommand};
use graph::TSGraph;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Transcript Segment Graph (TSG) CLI tool")]
struct Cli {
    /// Sets the level of verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Set verbosity level
    match cli.verbose {
        0 => tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init(),
        1 => tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init(),
        _ => tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init(),
    }

    match cli.command {
        Commands::Parse { input } => {
            info!("Parsing TSG file: {}", input.display());
            let graph = TSGraph::from_file(input)?;
            info!(
                "Successfully parsed TSG file with {} nodes and {} edges",
                graph.get_nodes().len(),
                graph.get_edges().len()
            );
            Ok(())
        }
        Commands::Dot { input, output } => {
            info!("Converting TSG file to DOT: {}", input.display());
            let graph = TSGraph::from_file(input)?;
            let dot = graph.to_dot()?;

            if let Some(output_path) = output {
                std::fs::write(&output_path, dot)?;
                info!("DOT file written to: {}", output_path.display());
            } else {
                println!("{}", dot);
            }
            Ok(())
        }

        Commands::Traverse { input, output } => {
            info!("Finding paths in TSG file: {}", input.display());
            let graph = TSGraph::from_file(input)?;
            let paths = graph.traverse()?;

            info!("Found {} paths", paths.len());

            if let Some(output_path) = output {
                let mut content = String::new();
                for path in paths {
                    content.push_str(&format!("{}\n", path));
                }
                std::fs::write(&output_path, content)?;
                info!("Paths written to: {}", output_path.display());
            } else {
                for path in paths {
                    println!("{}", path);
                }
            }
            Ok(())
        }
    }
}

fn main() -> Result<()> {
    run()?;
    Ok(())
}
