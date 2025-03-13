use anyhow::Result;
use std::path::Path;

use crate::graph::TSGraph;
use std::io::Write;

pub fn to_json<P: AsRef<Path>>(tsg_graph: &TSGraph, pretty: bool, output: P) -> Result<()> {
    for (id, graph) in tsg_graph.graphs.iter() {
        let graph_output_file = output.as_ref().with_extension(format!("{}.json", id));
        let output_file = std::fs::File::create(graph_output_file)?;
        let mut writer = std::io::BufWriter::new(output_file);
        let json = graph.to_json()?;
        if pretty {
            let json = serde_json::to_string_pretty(&json)?;
            writeln!(writer, "{}", json)?;
        } else {
            writeln!(writer, "{}", json)?;
        }
    }

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
