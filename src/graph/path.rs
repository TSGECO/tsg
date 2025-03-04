use std::fmt;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use bstr::BString;
use bstr::ByteVec;
use petgraph::graph::{EdgeIndex, NodeIndex};

use super::TSGraph;

/// A path in the transcript segment graph
///
/// A path is a sequence of nodes and edges that form a valid path through the graph.
/// Paths can represent transcripts, exon chains, or other traversals through the graph.
#[derive(Debug, Clone, Default)]
pub struct TSGPath {
    /// The nodes in the path
    pub nodes: Vec<NodeIndex>,
    /// The edges connecting the nodes in the path
    pub edges: Vec<EdgeIndex>,
    /// Optional identifier for the path
    id: Option<BString>,
}

impl fmt::Display for TSGPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nodes: Vec<String> = self
            .nodes
            .iter()
            .map(|&idx| idx.index().to_string())
            .collect();

        let edges: Vec<String> = self
            .edges
            .iter()
            .map(|&idx| idx.index().to_string())
            .collect();

        write!(f, "P\t{}\t{}", nodes.join(","), edges.join(","))
    }
}

impl TSGPath {
    /// Create a new empty path
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            id: None,
        }
    }

    /// Create a new path with the given ID
    pub fn with_id(id: &str) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            id: Some(id.into()),
        }
    }

    /// Add a node to the path
    pub fn add_node(&mut self, node: NodeIndex) {
        self.nodes.push(node);
    }

    /// Add an edge to the path
    pub fn add_edge(&mut self, edge: EdgeIndex) {
        self.edges.push(edge);
    }

    /// Get the number of nodes in the path
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges in the path
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check if the path is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    /// Set the ID of the path
    pub fn set_id(&mut self, id: &str) {
        self.id = Some(id.into());
    }

    pub fn get_id(&self) -> Option<&BString> {
        self.id.as_ref()
    }

    pub fn validate(&self) -> Result<()> {
        if self.nodes.len() != self.edges.len() + 1 {
            return Err(anyhow!("Invalid path: node count must be edge count + 1"));
        }
        Ok(())
    }

    pub fn to_gtf(&self, tsg_graph: &TSGraph) -> Result<BString> {
        todo!()
    }

    pub fn to_vcf(&self, tsg_graph: &TSGraph) -> Result<BString> {
        todo!()
    }

    pub fn to_fa(&self, tsg_graph: &TSGraph) -> Result<BString> {
        let mut seq = BString::from("");
        for node_idx in &self.nodes {
            let node_data = tsg_graph
                .get_node_by_idx(*node_idx)
                .context(format!("Node not found for index: {}", node_idx.index()))
                .unwrap();

            let node_seq = node_data
                .sequence
                .as_ref()
                .ok_or_else(|| anyhow!("Node sequence not found"))?;
            seq.push_str(node_seq);
        }
        Ok(seq)
    }
}
