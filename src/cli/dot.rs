use std::{io::Write, path::Path};

use crate::graph::TSGraph;
use anyhow::Result;

pub fn to_dot<P: AsRef<Path>>(input: P, output: Option<P>) -> Result<()> {
    let graph = TSGraph::from_file(input)?;

    for (id, graph) in graph.graphs.iter() {
        let graph_output_file = output
            .as_ref()
            .unwrap()
            .as_ref()
            .with_extension(format!("_{}", id));
        let output_file = std::fs::File::create(graph_output_file)?;
        let mut writer = std::io::BufWriter::new(output_file);
        let dot = graph.to_dot(true, true)?;
        writer.write_all(dot.as_bytes())?;
    }
    Ok(())
}
