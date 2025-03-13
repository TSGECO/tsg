use std::path::{Path, PathBuf};

use crate::graph::TSGraph;
use anyhow::{Result, anyhow};
use tracing::info;

/// Split a TSG file containing multiple graphs into multiple TSG files, each containing a single graph
///
/// This function takes a TSG file with multiple graphs and splits it into multiple TSG files,
/// where each output file contains a single graph from the original file.
/// The output files will be named based on the graph IDs.
pub fn split<P: AsRef<Path>>(input: P, output_dir: PathBuf) -> Result<()> {
    // Load the input TSG file
    info!("Loading TSG file: {}", input.as_ref().display());
    let tsg = TSGraph::from_file(input.as_ref())?;

    // Check if the output directory exists, create it if it doesn't
    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
        info!("Created output directory: {}", output_dir.display());
    }

    // Check if there are any graphs to split
    if tsg.graphs.is_empty() {
        return Err(anyhow!("No graphs found in the input TSG file"));
    }

    info!("Found {} graphs to split", tsg.graphs.len());

    // Process each graph in the input TSG
    for (graph_id, graph) in &tsg.graphs {
        // Create a new TSGraph for this single graph
        let mut single_graph_tsg = TSGraph::new();

        // Copy the headers from the original TSG
        single_graph_tsg.headers = tsg.headers.clone();

        // Add the current graph to the new TSGraph
        single_graph_tsg
            .graphs
            .insert(graph_id.clone(), graph.clone());

        // Filter links that are relevant to this graph
        for link in &tsg.links {
            if link.source_graph == *graph_id || link.target_graph == *graph_id {
                single_graph_tsg.links.push(link.clone());
            }
        }

        // Create the output file path
        let graph_id_str = graph_id.to_string();
        let output_file = output_dir.join(format!("{}.tsg", graph_id_str));

        // Write the single-graph TSG to a file
        info!(
            "Writing graph '{}' to: {}",
            graph_id_str,
            output_file.display()
        );
        single_graph_tsg.to_file(&output_file)?;
    }

    info!("Split completed successfully");
    Ok(())
}
