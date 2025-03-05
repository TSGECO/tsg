mod cli;
mod graph;
mod io;

use tracing::info;

use anyhow::Result;
use clap::{Command, CommandFactory, Parser};
use cli::Commands;
use graph::TSGraph;

use clap_complete::aot::{Generator, Shell, generate};
use std::io::stdout;

#[derive(Parser)]
#[command(author, version, about = "Transcript Segment Graph (TSG) CLI tool")]
#[command(propagate_version = true)]
struct Cli {
    // If provided, outputs the completion file for given shell
    #[arg(long = "generate", value_enum)]
    generator: Option<Shell>,

    /// Sets the level of verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    generate(generator, cmd, cmd.get_name().to_string(), &mut stdout());
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if let Some(generator) = cli.generator {
        let mut cmd = Cli::command();
        info!("Generating completion file for {generator:?}...");
        print_completions(generator, &mut cmd);
        return Ok(());
    }

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
            cli::to_dot(input, output)?;
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

        Commands::Fa {
            input,
            reference_genome,
            output,
        } => {
            info!("Converting TSG file to FASTA: {}", input.display());
            cli::to_fa(input, reference_genome, output)?;
            Ok(())
        }

        Commands::Gtf { input, output } => {
            info!("Converting TSG file to GTF: {}", input.display());
            cli::to_gtf(input, output)?;
            Ok(())
        }

        Commands::Vcf { input, output } => {
            info!("Converting TSG file to VCF: {}", input.display());
            cli::to_vcf(input, output)?;
            Ok(())
        }

        Commands::Json {
            input,
            pretty,
            output,
        } => {
            info!("Converting TSG file to JSON: {}", input.display());
            cli::to_json(input, pretty, output)?;
            Ok(())
        }
    }
}

fn main() -> Result<()> {
    run()?;
    Ok(())
}
