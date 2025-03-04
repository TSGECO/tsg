use std::path::Path;

use crate::graph::TSGraph;
use anyhow::Result;
use tracing::info;

pub fn to_dot<P: AsRef<Path>>(input: P, output: Option<P>) -> Result<()> {
    info!("Converting TSG file to DOT: {}", input.as_ref().display());
    let graph = TSGraph::from_file(input)?;
    let dot = graph.to_dot()?;

    if let Some(output_path) = output {
        std::fs::write(&output_path, dot)?;
        info!("DOT file written to: {}", output_path.as_ref().display());
    } else {
        println!("{}", dot);
    }
    Ok(())
}
