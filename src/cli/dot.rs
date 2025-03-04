use std::{io::Write, path::Path};

use crate::graph::TSGraph;
use anyhow::Result;

pub fn to_dot<P: AsRef<Path>>(input: P, output: Option<P>) -> Result<()> {
    let graph = TSGraph::from_file(input)?;
    let dot = graph.to_dot(true, true)?;

    if let Some(output_path) = output {
        std::fs::write(&output_path, dot)?;
    } else {
        // write to stdout
        let stdout = std::io::stdout();
        let mut writer = std::io::BufWriter::new(stdout.lock());
        writer.write_all(dot.as_bytes())?;
    }
    Ok(())
}
