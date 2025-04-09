use crate::graph::TSGraph;
use anyhow::Result;
use bstr::BString;
use petgraph::graph::NodeIndex;

// TODO this module is not used yet, but it will be used in the future
#[allow(dead_code)]
pub trait GraphAnalysis {
    fn is_connected(&self) -> bool;
    fn is_cyclic(&self) -> bool;
    fn detect_bubbles(&self) -> Vec<Vec<NodeIndex>>;
    fn is_directed_acyclic_graph(&self) -> bool {
        self.is_connected() && !self.is_cyclic()
    }
    fn summarize(&self) -> Result<BString>;
}

impl GraphAnalysis for TSGraph {
    fn is_connected(&self) -> bool {
        // Implementation here
        unimplemented!()
    }

    fn is_cyclic(&self) -> bool {
        // Implementation here
        unimplemented!()
    }

    fn detect_bubbles(&self) -> Vec<Vec<NodeIndex>> {
        // Implementation here
        unimplemented!()
    }

    fn summarize(&self) -> Result<BString> {
        // Pre-calculate capacity based on expected size
        let graph_count = self.graphs.len();
        // Estimate 30 bytes per graph entry (adjust as needed based on actual data sizes)
        let estimated_capacity = graph_count * 30;

        // Pre-allocate with capacity to avoid reallocations
        let mut summary = Vec::with_capacity(estimated_capacity);

        let header = b"gid\tnodes\tedges\tpaths\tmax_path_len\n";
        summary.extend_from_slice(header);

        for (id, graph) in self.graphs.iter() {
            let node_count = graph.nodes().len();
            let edge_count = graph.edges().len();
            let paths = graph.traverse()?;

            let path_count = paths.len();
            let max_path_len = paths.iter().map(|path| path.nodes.len()).max().unwrap_or(0);

            // Use write! to format directly into the buffer without intermediate allocations
            use std::io::Write;
            writeln!(
                summary,
                "{}\t{}\t{}\t{}\t{}",
                id, node_count, edge_count, path_count, max_path_len
            )?;
        }

        // Convert to BString only once at the end
        Ok(BString::from(summary))
    }
}
