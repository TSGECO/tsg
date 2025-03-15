use std::io::Write;
use std::path::Path;

use anyhow::{Result, anyhow};
use bstr::ByteSlice;
use tracing::info;
use tsg::graph::TSGraph;

/// Merge multiple TSG files into a single TSG file
///
/// This function takes multiple TSG files and merges them into a single TSG file.
/// The merged TSG will contain all graphs from all input files, with unique graph IDs.
/// If there are duplicate graph IDs, they will be renamed with a suffix.
pub fn merge<P: AsRef<Path>>(inputs: Vec<P>, output: Option<P>) -> Result<()> {
    if inputs.is_empty() {
        return Err(anyhow!("No input files provided"));
    }
    info!("Merging {} TSG files", inputs.len());

    // Create a new empty TSG to hold the merged result
    let mut merged_tsg = TSGraph::new();

    // Process each input file
    for (idx, input) in inputs.iter().enumerate() {
        info!("Processing file {}: {}", idx + 1, input.as_ref().display());

        // Load the TSG file
        let tsg = TSGraph::from_file(input.as_ref())?;

        // Merge headers (avoiding duplicates)
        for header in tsg.headers {
            if !merged_tsg
                .headers
                .iter()
                .any(|h| h.tag == header.tag && h.value == header.value)
            {
                merged_tsg.headers.push(header);
            }
        }

        // Merge graphs (handling potential ID conflicts)
        for (graph_id, graph) in tsg.graphs {
            let mut new_id = graph_id.clone();

            // If this graph ID already exists in the merged TSG, create a unique ID
            if merged_tsg.graphs.contains_key(&graph_id) {
                let new_id_str = format!("{}_{}", graph_id.to_str().unwrap_or("graph"), idx);
                info!(
                    "Renamed duplicate graph ID '{}' to '{}'",
                    graph_id.to_str().unwrap_or("unknown"),
                    &new_id_str
                );
                new_id = new_id_str.into();
            }

            // Add the graph to the merged TSG
            merged_tsg.graphs.insert(new_id, graph);
        }

        // Merge inter-graph links
        for link in tsg.links {
            merged_tsg.links.push(link);
        }
    }

    // Write the merged TSG to the output file

    let mut writer: Box<dyn Write> = match output {
        Some(path) => {
            info!("Writing paths to file: {}", path.as_ref().display());
            Box::new(std::io::BufWriter::new(std::fs::File::create(path)?))
        }
        None => {
            info!("Writing paths to stdout");
            Box::new(std::io::BufWriter::new(std::io::stdout().lock()))
        }
    };

    merged_tsg.to_writer(&mut writer)?;
    info!("Merge completed successfully");
    Ok(())
}
