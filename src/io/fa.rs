use std::path::Path;

use crate::graph::TSGraph;
use anyhow::Result;
use std::io::Write;

pub fn to_fa<P: AsRef<Path>, Q: AsRef<Path>>(
    tsg_graph: &mut TSGraph,
    reference_genome_path: P,
    output: Q,
) -> Result<()> {
    tsg_graph.annotate_node_with_sequence(reference_genome_path)?;
    let paths = tsg_graph.traverse()?;

    let output_file = std::fs::File::create(output)?;
    let mut writer = std::io::BufWriter::new(output_file);

    for path in paths {
        let seq = path.to_fa(tsg_graph)?;
        writeln!(writer, ">{}", path.get_id().unwrap())?;
        writeln!(writer, "{}", seq)?;
    }
    Ok(())
}
