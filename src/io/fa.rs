use std::path::Path;

use crate::graph::TSGraph;
use anyhow::Result;
use std::io::Write;

pub fn to_fa<P: AsRef<Path>>(tsg_graph: &mut TSGraph, output: P) -> Result<()> {
    let paths = tsg_graph.traverse_all_graphs()?;
    let output_file = std::fs::File::create(output)?;
    let mut writer = std::io::BufWriter::new(output_file);

    for path in paths {
        let seq = path.to_fa()?;
        writeln!(writer, ">{}", path.id().unwrap())?;
        writeln!(writer, "{}", seq)?;
    }
    Ok(())
}
