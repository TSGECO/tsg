use crate::graph::TSGraph;
use anyhow::Result;
use std::io::Write;

pub fn to_fa<W: Write>(tsg_graph: &mut TSGraph, writer: &mut W) -> Result<()> {
    let paths = tsg_graph.traverse_all_graphs()?;

    for path in paths {
        let seq = path.to_fa()?;
        writeln!(writer, ">{}", path.id().unwrap())?;
        writeln!(writer, "{}", seq)?;
    }
    Ok(())
}
