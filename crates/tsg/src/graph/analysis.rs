use crate::graph::TSGraph;
use petgraph::graph::NodeIndex;

pub trait GraphAnalysis {
    fn is_connected(&self) -> bool;
    fn is_cyclic(&self) -> bool;
    fn detect_bubbles(&self) -> Vec<Vec<NodeIndex>>;

    fn is_directed_acyclic_graph(&self) -> bool {
        self.is_connected() && !self.is_cyclic()
    }
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
}
