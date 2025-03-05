use std::fmt;

use super::Attribute;
use super::TSGraph;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use bon::Builder;
use bstr::BString;
use bstr::ByteSlice;
use bstr::ByteVec;
use petgraph::graph::{EdgeIndex, NodeIndex};

/// A path in the transcript segment graph
///
/// A path is a sequence of nodes and edges that form a valid path through the graph.
/// Paths can represent transcripts, exon chains, or other traversals through the graph.
#[derive(Debug, Clone, Default, Builder)]
pub struct TSGPath<'a> {
    /// The nodes in the path
    #[builder(default)]
    pub nodes: Vec<NodeIndex>,
    /// The edges connecting the nodes in the path
    #[builder(default)]
    pub edges: Vec<EdgeIndex>,
    /// Optional identifier for the path
    id: Option<BString>,
    graph: Option<&'a TSGraph>,

    #[builder(default)]
    pub attributes: Vec<Attribute>,
}

impl<'a> fmt::Display for TSGPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // O  transcript2  n2+  e3+  n3+  e4+  n5+
        // O  path_id n1+  e1+  n2+  e2+  n3+
        let mut res = vec![];
        res.push("O".to_string());
        res.push(self.id.clone().unwrap().to_string());
        for (idx, node_idx) in self.nodes.iter().enumerate() {
            let node_data = self
                .graph
                .ok_or_else(|| anyhow!("Graph not available"))
                .unwrap()
                .get_node_by_idx(*node_idx)
                .context(format!("Node not found for index: {}", node_idx.index()))
                .unwrap();

            let node_id = &node_data.id;
            res.push(format!("{}+", node_id));
            if idx < self.nodes.len() - 1 {
                let edge_data = self
                    .graph
                    .ok_or_else(|| anyhow!("Graph not available"))
                    .unwrap()
                    .get_edge_by_idx(self.edges[idx])
                    .context(format!(
                        "Edge not found for index: {}",
                        self.edges[idx].index()
                    ))
                    .unwrap();
                res.push(format!("{}+", edge_data.id));
            }
        }
        write!(f, "{}", res.join("\t"))
    }
}

impl<'a> TSGPath<'a> {
    /// Create a new empty path
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the graph for the path
    pub fn set_graph(&mut self, graph: &'a TSGraph) {
        self.graph = Some(graph);
    }

    pub fn get_graph(&self) -> Option<&TSGraph> {
        self.graph
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

    pub fn to_gtf(&self) -> Result<BString> {
        let id = if let Some(id) = self.id.as_ref() {
            id
        } else {
            return Err(anyhow!("Path ID not found"));
        };
        let mut transcript = format!(
            ".\ttsg\ttranscript\t.\t.\t.\t.\t.\ttranscript_id \"{}\";",
            id
        );

        for attr in &self.attributes {
            let attr_str = format!("{} \"{}\";", attr.tag, attr.value);
            transcript.push_str(&attr_str);
        }

        // create a hashmap of attributes
        let mut attributes = vec![
            Attribute::builder()
                .tag("transcript_id")
                .value(id.clone())
                .build(),
        ];

        let mut exons: Vec<BString> = vec![transcript.into()];

        for (idx, node_idx) in self.nodes.iter().enumerate() {
            let graph = self.graph.ok_or_else(|| anyhow!("Graph not available"))?;
            let node_data = graph
                .get_node_by_idx(*node_idx)
                .with_context(|| format!("Node not found for index: {}", node_idx.index()))?;

            attributes.push(
                Attribute::builder()
                    .tag("segment_id")
                    .value(format!("{:03}", idx))
                    .build(),
            );
            let exon = node_data.to_gtf(Some(&attributes))?;
            exons.push(exon);
        }

        // Convert Vec<BString> to a format that can be joined
        let exon_strs: Vec<&str> = exons.iter().map(|b| b.to_str().unwrap()).collect();
        Ok(exon_strs.join("\n").into())
    }

    pub fn to_vcf(&self) -> Result<BString> {
        let _id = if let Some(id) = self.id.as_ref() {
            id
        } else {
            return Err(anyhow!("Path ID not found"));
        };
        let mut edges = vec![];

        for edge_idx in self.edges.iter() {
            let graph = self.graph.ok_or_else(|| anyhow!("Graph not available"))?;
            let edge_data = graph
                .get_edge_by_idx(*edge_idx)
                .with_context(|| format!("Edge not found for index: {}", edge_idx.index()))?;

            let edge_vcf = edge_data.to_vcf(None)?;
            edges.push(edge_vcf);
        }

        let edge_strs: Vec<&str> = edges.iter().map(|b| b.to_str().unwrap()).collect();
        Ok(edge_strs.join("\n").into())
    }

    pub fn to_fa(&self) -> Result<BString> {
        let mut seq = BString::from("");
        for node_idx in &self.nodes {
            let node_data = self
                .graph
                .ok_or_else(|| anyhow!("Graph not available"))
                .unwrap()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_creation_and_accessors() {
        let path = TSGPath::new();
        assert!(path.is_empty());
        assert_eq!(path.node_count(), 0);
        assert_eq!(path.edge_count(), 0);
        assert_eq!(path.get_id(), None);
        assert!(path.get_graph().is_none());
    }
}
