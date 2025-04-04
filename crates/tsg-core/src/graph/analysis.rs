use crate::graph::TSGraph;
use anyhow::Result;
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
    fn summarize(&self) -> Result<()>;
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

    fn summarize(&self) -> Result<()> {
        // Implementation here
        unimplemented!()
    }
}
