use anyhow::Result;
use std::path::Path;

use crate::graph::TSGraph;
use std::io::Write;

pub fn to_json<P: AsRef<Path>>(tsg_graph: &TSGraph, output: P) -> Result<()> {
    let output_file = std::fs::File::create(output)?;
    let mut writer = std::io::BufWriter::new(output_file);
    let json = tsg_graph.to_json()?;
    // write json to file
    writeln!(writer, "{}", json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_gtf() {
        let tsg_graph = TSGraph::from_file("tests/data/test.tsg").unwrap();
        let output = "tests/data/test.json";
        to_json(&tsg_graph, output).unwrap();
    }
}
