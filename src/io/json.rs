use anyhow::Result;
use std::path::Path;

use crate::graph::TSGraph;
use std::io::Write;

pub fn to_json<P: AsRef<Path>>(tsg_graph: &TSGraph, pretty: bool, output: P) -> Result<()> {
    let output_file = std::fs::File::create(output)?;
    let mut writer = std::io::BufWriter::new(output_file);
    let json = tsg_graph.to_json()?;

    if pretty {
        let json = serde_json::to_string_pretty(&json)?;
        writeln!(writer, "{}", json)?;
        return Ok(());
    }
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
        to_json(&tsg_graph, true, output).unwrap();
    }
}
