use anyhow::Result;

use crate::graph::TSGraph;
use std::io::Write;

pub fn to_gtf<W: Write>(tsg_graph: &TSGraph, writer: &mut W) -> Result<()> {
    let paths = tsg_graph.traverse_all_graphs()?;
    for path in paths {
        let seq = path.to_gtf()?;
        writeln!(writer, "{}", seq)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_gtf() {
        let tsg_graph = TSGraph::from_file("tests/data/test.tsg").unwrap();
        let output = "tests/data/test.gtf";

        let file = std::fs::File::create(output).unwrap();
        let mut writer = std::io::BufWriter::new(file);
        to_gtf(&tsg_graph, &mut writer).unwrap();
    }
}
