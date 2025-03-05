use anyhow::Result;
use std::path::Path;

use crate::graph::TSGraph;
use std::io::Write;

pub fn to_gtf<P: AsRef<Path>>(tsg_graph: &TSGraph, output: P) -> Result<()> {
    let paths = tsg_graph.traverse()?;
    let output_file = std::fs::File::create(output)?;
    let mut writer = std::io::BufWriter::new(output_file);

    for path in paths {
        let seq = path.to_gtf()?;
        write!(writer, "{}\n", seq)?;
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
        to_gtf(&tsg_graph, output).unwrap();
    }
}
