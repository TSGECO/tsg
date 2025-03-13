use crate::graph::TSGraph;
use anyhow::{Result, anyhow};
use bstr::BString;
use clap::{Args, Subcommand};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// Path to the TSG file
    #[clap(short, long)]
    pub input: PathBuf,

    #[clap(subcommand)]
    pub command: QueryCommands,
}

#[derive(Subcommand, Debug)]
pub enum QueryCommands {
    /// Get specific graphs by ID
    Graph(GraphQueryArgs),
}

#[derive(Args, Debug)]
pub struct GraphQueryArgs {
    /// Graph IDs to query (can specify multiple)
    #[clap(short, long, value_delimiter = ',')]
    pub ids: Vec<String>,

    /// Path to a file containing graph IDs (one ID per line)
    #[clap(short, long)]
    pub ids_file: Option<PathBuf>,

    /// Output format (dot, json)
    #[clap(short, long, default_value = "json")]
    pub format: String,

    /// Include node labels in dot output
    #[clap(long, default_value = "false")]
    pub node_labels: bool,

    /// Include edge labels in dot output
    #[clap(long, default_value = "false")]
    pub edge_labels: bool,

    /// Output file path (if not specified, prints to stdout)
    #[clap(short, long)]
    pub output: Option<PathBuf>,

    /// Append to output file instead of overwriting
    #[clap(short, long, default_value = "false")]
    pub append: bool,
}

pub fn execute_query(args: &QueryArgs) -> Result<()> {
    let tsg = TSGraph::from_file(&args.input)?;

    match &args.command {
        QueryCommands::Graph(graph_args) => query_graphs(&tsg, graph_args),
    }
}

fn query_graphs(tsg: &TSGraph, args: &GraphQueryArgs) -> Result<()> {
    // Collect all graph IDs to query
    let mut graph_ids = args.ids.clone();

    // If an IDs file is provided, read IDs from the file (one per line)
    if let Some(ids_file_path) = &args.ids_file {
        let file =
            File::open(ids_file_path).map_err(|e| anyhow!("Failed to open IDs file: {}", e))?;

        let reader = BufReader::new(file);
        for line in reader.lines() {
            let id = line?;
            if !id.trim().is_empty() {
                graph_ids.push(id.trim().to_string());
            }
        }
    }

    if graph_ids.is_empty() {
        return Err(anyhow!("No graph IDs specified"));
    }

    let mut results = Vec::new();

    // Process each requested graph ID
    for id in &graph_ids {
        // Check if the graph exists
        if !tsg.graphs.contains_key(&BString::from(id.as_bytes())) {
            return Err(anyhow!("Graph with ID '{}' not found", id));
        }

        let result = match args.format.as_str() {
            "dot" => {
                let dot = tsg.to_dot_by_id(id, args.node_labels, args.edge_labels)?;
                format!("# Graph: {}\n{}\n", id, dot)
            }
            "json" => {
                let json = tsg.to_json_by_id(id)?;
                format!(
                    "# Graph: {}\n{}\n",
                    id,
                    serde_json::to_string_pretty(&json)?
                )
            }
            _ => return Err(anyhow!("Unsupported output format: {}", args.format)),
        };

        results.push(result);
    }

    // Combine all results
    let combined_result = results.join("\n");

    // Output the result
    if let Some(output_path) = &args.output {
        if args.append {
            use std::fs::OpenOptions;
            use std::io::Write;

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(output_path)?;

            writeln!(file, "{}", combined_result)?;
        } else {
            std::fs::write(output_path, combined_result)?;
        }
    } else {
        println!("{}", combined_result);
    }

    Ok(())
}
