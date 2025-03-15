use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use bstr::BString;
use tracing::info;

use tsg::graph::TSGraph;

/// Query specific graphs from a TSG file
///
/// This function extracts specific graphs by their IDs from a TSG file
/// and outputs them in the specified format.
pub fn query(
    input: PathBuf,
    ids_str: String,
    ids_file: Option<PathBuf>,
    output: Option<PathBuf>,
) -> Result<()> {
    info!("Querying graphs from TSG file: {}", input.display());
    let tsg = TSGraph::from_file(&input)?;

    // Collect all graph IDs to query
    let mut graph_ids = ids_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>();

    // If an IDs file is provided, read IDs from the file (one per line)
    if let Some(ids_file_path) = ids_file {
        info!("Reading graph IDs from file: {}", ids_file_path.display());
        let file =
            File::open(&ids_file_path).map_err(|e| anyhow!("Failed to open IDs file: {}", e))?;

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

    info!("Querying {} graphs", graph_ids.len());

    // Create a new TSGraph to hold the queried graphs
    let mut queried_tsg = TSGraph::new();

    // Copy headers from the original TSG
    queried_tsg.headers = tsg.headers.clone();

    // Process each requested graph ID
    for id in &graph_ids {
        let bstring_id = BString::from(id.as_bytes());

        // Check if the graph exists
        if !tsg.graphs.contains_key(&bstring_id) {
            return Err(anyhow!("Graph with ID '{}' not found", id));
        }

        // Copy the graph to the new TSG
        if let Some(graph) = tsg.graphs.get(&bstring_id) {
            queried_tsg.graphs.insert(bstring_id.clone(), graph.clone());

            // Copy relevant links
            for link in &tsg.links {
                if link.source_graph == bstring_id || link.target_graph == bstring_id {
                    queried_tsg.links.push(link.clone());
                }
            }
        }
    }

    // Output the result
    if let Some(output_path) = output {
        info!("Writing queried graphs to: {}", output_path.display());
        queried_tsg.to_file(&output_path)?;
    } else {
        // Print to stdout in TSG format
        let stdout = std::io::stdout();
        let mut writer = std::io::BufWriter::new(stdout.lock());
        queried_tsg.to_writer(&mut writer)?;
    }

    info!("Query completed successfully");
    Ok(())
}
