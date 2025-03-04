use crate::graph::TSGraph;
use anyhow::Result;

pub fn annotate_node_with_sequence<P: AsRef<P>>(
    tsg_graph: &TSGraph,
    reference_genome_path: P,
) -> Result<()> {
    todo!()
}

pub fn to_fa<P: AsRef<P>>(tsg_graph: &TSGraph, reference_genome_path: P, output: P) -> Result<()> {
    annotate_node_with_sequence(tsg_graph, reference_genome_path)?;
    todo!()
}
