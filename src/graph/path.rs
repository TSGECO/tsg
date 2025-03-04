use std::fmt;

use anyhow::Result;
use anyhow::anyhow;
use bstr::{BStr, BString, ByteSlice};
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
    pub id: Option<BString>,
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
    pub fn with_id<S: Into<BString>>(id: S) -> Self {
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
    pub fn set_id<S: Into<BString>>(&mut self, id: S) {
        self.id = Some(id.into());
    }

    /// Get the ID of the path, if it has one
    pub fn id(&self) -> Option<&BStr> {
        self.id.as_ref().map(|s| s.as_bstr())
    }

    pub fn validate(&self) -> Result<()> {
        if self.nodes.len() != self.edges.len() + 1 {
            return Err(anyhow!("Invalid path: node count must be edge count + 1"));
        }

        Ok(())
    }

    pub fn to_gtf(&self, tsg_graph: &TSGraph, path_id: usize) -> Result<BString> {
        todo!()
    }

    pub fn to_vcf(&self, tsg_graph: &TSGraph, path_id: usize) -> Result<BString> {
        todo!()
    }

    pub fn to_fa<P: AsRef<P>>(
        &self,
        tsg_graph: &TSGraph,
        path_id: usize,
        reference_path: P,
    ) -> Result<BString> {
        todo!()
    }
}
