mod edge;
mod group;
mod node;
mod read;

use std::fmt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;

use ahash::{HashMap, HashMapExt};
use bstr::BStr;
use bstr::ByteSlice;
pub use edge::*;
pub use group::*;
pub use node::*;
pub use read::*;

use anyhow::Result;
use anyhow::anyhow;
use bstr::BString;

use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};

/// Represents an optional attribute
#[derive(Debug, Clone)]
pub struct Attribute {
    pub tag: BString,
    pub attribute_type: char,
    pub value: BString,
}

impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\t{}\t{}", self.tag, self.attribute_type, self.value)
    }
}

/// Header information in the TSG file
#[derive(Debug, Clone)]
pub struct Header {
    pub tag: BString,
    pub value: BString,
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "H\t{}\t{}", self.tag, self.value)
    }
}

/// The complete transcript segment graph
#[derive(Debug, Clone, Default)]
pub struct TSGraph {
    pub headers: Vec<Header>,
    _graph: DiGraph<NodeData, EdgeData>,
    pub node_indices: HashMap<BString, NodeIndex>,
    pub edge_indices: HashMap<BString, EdgeIndex>,
    pub groups: HashMap<BString, Group>,
    pub chains: HashMap<BString, Group>, // We store chains separately but they're also in groups
}

impl TSGraph {
    /// Create a new empty TSGraph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node_data: NodeData) -> Result<NodeIndex> {
        let id = node_data.id.clone();

        // Check if node already exists
        if let Some(&idx) = self.node_indices.get(&id) {
            // Update node data
            if let Some(attr) = self._graph.node_weight_mut(idx) {
                *attr = node_data;
            } else {
                return Err(anyhow!("Node with ID {} not found in graph", id));
            }
            return Ok(idx);
        }

        // Add new node
        let node_idx = self._graph.add_node(node_data);
        self.node_indices.insert(id, node_idx);
        Ok(node_idx)
    }

    /// Add an edge to the graph
    pub fn add_edge(
        &mut self,
        source_id: &BStr,
        sink_id: &BStr,
        edge_data: EdgeData,
    ) -> Result<EdgeIndex> {
        let id = edge_data.id.clone();

        // Get source and sink node indices
        let source_idx = *self
            .node_indices
            .get(source_id.as_bytes())
            .ok_or_else(|| anyhow!("Source node with ID {} not found", source_id))?;

        let sink_idx = *self
            .node_indices
            .get(sink_id.as_bytes())
            .ok_or_else(|| anyhow!("Sink node with ID {} not found", sink_id))?;

        let edge_idx = self._graph.update_edge(source_idx, sink_idx, edge_data);

        self.edge_indices.insert(id, edge_idx);
        Ok(edge_idx)
    }

    /// Parse a header line
    fn parse_header_line(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() < 3 {
            return Err(anyhow!("Invalid header line format"));
        }

        self.headers.push(Header {
            tag: fields[1].into(),
            value: fields[2].into(),
        });

        Ok(())
    }

    /// Parse a node line
    fn parse_node_line(&mut self, fields: &str) -> Result<()> {
        let node_data = NodeData::from_str(fields)?;
        self.add_node(node_data)?;
        Ok(())
    }

    /// Parse an edge line
    fn parse_edge_line(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() < 5 {
            return Err(anyhow!("Invalid edge line format"));
        }

        let id: BString = fields[1].into();
        let source_id: BString = fields[2].into();
        let sink_id: BString = fields[3].into();
        let sv = fields[4].parse::<StructuralVariant>()?;

        let edge_data = EdgeData {
            id,
            sv,
            ..Default::default()
        };

        self.add_edge(source_id.as_bstr(), sink_id.as_bstr(), edge_data)?;
        Ok(())
    }

    /// Parse an unordered group line
    fn parse_unordered_group_line(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() < 3 {
            return Err(anyhow!("Invalid unordered group line format"));
        }

        let id: BString = fields[1].into();

        // Check for duplicate group name
        if self.groups.contains_key(&id) {
            return Err(anyhow!("Group with ID {} already exists", id));
        }

        // Parse element IDs (space-separated)
        let elements_str = fields[2..].join(" ");
        let elements = elements_str
            .split_whitespace()
            .map(|s| s.into())
            .collect::<Vec<_>>();

        let group = Group::Unordered {
            id: id.clone(),
            elements,
            attributes: HashMap::new(),
        };

        self.groups.insert(id, group);
        Ok(())
    }

    /// Parse an ordered group line
    fn parse_ordered_group_line(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() < 3 {
            return Err(anyhow!("Invalid ordered group line format"));
        }

        let id: BString = fields[1].into();

        // Check for duplicate group name
        if self.groups.contains_key(&id) {
            return Err(anyhow!(format!("Group with ID {} already exists", id)));
        }

        // Parse oriented element IDs (space-separated)
        let elements_str = fields[2..].join(" ");
        let elements = elements_str
            .split_whitespace()
            .map(|s| s.parse::<OrientedElement>())
            .collect::<Result<Vec<_>, _>>()?;

        let group = Group::Ordered {
            id: id.clone(),
            elements,
            attributes: HashMap::new(),
        };

        self.groups.insert(id, group);
        Ok(())
    }

    /// Parse a chain line
    fn parse_chain_line(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() < 3 {
            return Err(anyhow!("Invalid chain line format"));
        }

        let id: BString = fields[1].into();

        // Check for duplicate group name
        if self.groups.contains_key(&id) {
            return Err(anyhow!("Group with ID {} already exists", id));
        }

        // Parse element IDs (space-separated)
        let elements_str = fields[2..].join(" ");
        let elements: Vec<BString> = elements_str.split_whitespace().map(|s| s.into()).collect();

        // Validate chain structure: must start and end with nodes, and have alternating nodes/edges
        if elements.is_empty() {
            return Err(anyhow!("Chain must contain at least one element"));
        }

        if elements.len() % 2 == 0 {
            return Err(anyhow!(
                "Chain must have an odd number of elements (starting and ending with nodes)"
            ));
        }

        // Create the chain group
        let group = Group::Chain {
            id: id.clone(),
            elements,
            attributes: HashMap::new(),
        };

        // Store the chain in both maps
        self.chains.insert(id.clone(), group.clone());
        self.groups.insert(id, group);
        Ok(())
    }

    /// Parse an attribute line
    fn parse_attribute_line(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() < 6 {
            return Err(anyhow!("Invalid attribute line format"));
        }

        let element_type = fields[1];
        let element_id: BString = fields[2].into();
        let tag: BString = fields[3].into();
        let attr_type = fields[4]
            .chars()
            .next()
            .ok_or_else(|| anyhow!("Invalid attribute type character"))?;
        let value = fields[5].into();

        let attribute = Attribute {
            tag: tag.clone(),
            attribute_type: attr_type,
            value,
        };

        match element_type {
            "N" => {
                if let Some(&node_idx) = self.node_indices.get(&element_id) {
                    // A  <element_type>  <element_id>  <tag>  <type>  <value>
                    if let Some(node_data) = self._graph.node_weight_mut(node_idx) {
                        node_data.attributes.insert(tag, attribute);
                    } else {
                        return Err(anyhow!("Node with ID {} not found in graph", element_id));
                    }
                } else {
                    return Err(anyhow!("Node with ID {} not found", element_id));
                }
            }
            "E" => {
                if let Some(&edge_idx) = self.edge_indices.get(&element_id) {
                    if let Some(edge_data) = self._graph.edge_weight_mut(edge_idx) {
                        edge_data.attributes.insert(tag, attribute);
                    } else {
                        return Err(anyhow!("Edge with ID {} not found in graph", element_id));
                    }
                } else {
                    return Err(anyhow!("Edge with ID {} not found", element_id));
                }
            }
            "U" | "O" | "C" => {
                if let Some(group) = self.groups.get_mut(&element_id) {
                    match group {
                        Group::Unordered { attributes, .. } => {
                            attributes.insert(tag, attribute);
                        }
                        Group::Ordered { attributes, .. } => {
                            attributes.insert(tag, attribute);
                        }
                        Group::Chain { attributes, .. } => {
                            attributes.insert(tag, attribute);
                        }
                    }
                } else {
                    return Err(anyhow!("Group with ID {} not found", element_id));
                }
            }
            _ => {
                return Err(anyhow!("Unknown element type: {}", element_type));
            }
        }

        Ok(())
    }

    /// Build graph based on the current TSG state
    fn ensure_graph_is_built(&mut self) -> Result<()> {
        // If we already have nodes and edges, assume the graph is properly constructed
        if !self.node_indices.is_empty() && !self.edge_indices.is_empty() {
            return Ok(());
        }

        // If nodes and edges are missing, build them from chains
        if !self.chains.is_empty() {
            for (_, group) in &self.chains.clone() {
                if let Group::Chain { elements, .. } = group {
                    // Process each element in the chain
                    for (i, element_id) in elements.iter().enumerate() {
                        if i % 2 == 0 {
                            // It's a node - add it if it doesn't exist
                            if !self.node_indices.contains_key(element_id) {
                                // Create placeholder node
                                let node_data = NodeData {
                                    id: element_id.clone(),
                                    ..Default::default()
                                };
                                self.add_node(node_data)?;
                            }
                        } else {
                            // It's an edge - add it if it doesn't exist
                            if !self.edge_indices.contains_key(element_id) {
                                // Get connecting nodes
                                let source_id = &elements[i - 1];
                                let sink_id = &elements[i + 1];

                                // Create placeholder edge
                                let edge_data = EdgeData {
                                    id: element_id.clone(),
                                    ..Default::default()
                                };

                                if let Err(e) =
                                    self.add_edge(source_id.as_bstr(), sink_id.as_bstr(), edge_data)
                                {
                                    return Err(anyhow!("Failed to add edge from chain: {}", e));
                                }
                            }
                        }
                    }
                }
            }
            return Ok(());
        }

        // If we have neither explicit nodes/edges nor chains, that's an error
        if self.node_indices.is_empty() || self.edge_indices.is_empty() {
            return Err(anyhow!(
                "Cannot build graph: no nodes/edges defined and no chains available"
            ));
        }

        Ok(())
    }

    /// Validate paths against the graph
    fn validate_paths(&self) -> Result<()> {
        for (id, group) in &self.groups {
            if let Group::Ordered { elements, .. } = group {
                // Validate that all elements in the path exist in the graph
                for element in elements {
                    let element_exists = self.node_indices.contains_key(&element.id)
                        || self.edge_indices.contains_key(&element.id)
                        || self.groups.contains_key(&element.id);

                    if !element_exists {
                        return Err(anyhow!(
                            "Path {} references non-existent element {}",
                            id,
                            element.id
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse a TSG file and construct a TSGraph
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut tsgraph = TSGraph::new();

        // First pass: Parse all record types
        for line in reader.lines() {
            let line = line?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();
            if fields.is_empty() {
                continue;
            }

            match fields[0] {
                "H" => tsgraph.parse_header_line(&fields)?,
                "N" => tsgraph.parse_node_line(&line)?,
                "E" => tsgraph.parse_edge_line(&fields)?,
                "U" => tsgraph.parse_unordered_group_line(&fields)?,
                "O" => tsgraph.parse_ordered_group_line(&fields)?,
                "C" => tsgraph.parse_chain_line(&fields)?,
                "A" => tsgraph.parse_attribute_line(&fields)?,
                _ => {
                    return Err(anyhow!("Unknown record type: {}", fields[0]));
                }
            }
        }

        // Populate chains hash map from groups if needed
        for (id, group) in &tsgraph.groups {
            if let Group::Chain { .. } = group {
                if !tsgraph.chains.contains_key(id) {
                    tsgraph.chains.insert(id.clone(), group.clone());
                }
            }
        }

        // Second pass: Ensure the graph is built (if needed) and validate paths
        tsgraph.ensure_graph_is_built()?;
        tsgraph.validate_paths()?;

        Ok(tsgraph)
    }

    /// Get the nodes in a chain in order
    pub fn get_chain_nodes(&self, chain_id: &BStr) -> Option<Vec<NodeIndex>> {
        let group = self.chains.get(chain_id)?;

        match group {
            Group::Chain { elements, .. } => {
                let mut nodes = Vec::new();

                for (i, element_id) in elements.iter().enumerate() {
                    if i % 2 == 0 {
                        // Nodes are at even positions (0, 2, 4...)
                        if let Some(&node_idx) = self.node_indices.get(element_id) {
                            nodes.push(node_idx);
                        } else {
                            return None; // Invalid chain structure
                        }
                    }
                }

                Some(nodes)
            }
            _ => None, // Not a chain
        }
    }

    /// Get the edges in a chain in order
    pub fn get_chain_edges(&self, chain_id: &BStr) -> Option<Vec<EdgeIndex>> {
        let group = self.chains.get(chain_id)?;

        match group {
            Group::Chain { elements, .. } => {
                let mut edges = Vec::new();

                for (i, element_id) in elements.iter().enumerate() {
                    if i % 2 == 1 {
                        // Edges are at odd positions (1, 3, 5...)
                        if let Some(&edge_idx) = self.edge_indices.get(element_id) {
                            edges.push(edge_idx);
                        } else {
                            return None; // Invalid chain structure
                        }
                    }
                }

                Some(edges)
            }
            _ => None, // Not a chain
        }
    }

    /// Write the TSGraph to a TSG file
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        use std::fs::File;
        use std::io::{BufWriter, Write};

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write headers
        for header in &self.headers {
            writeln!(writer, "{}", header)?;
        }

        // Write nodes
        for node_idx in self._graph.node_indices() {
            if let Some(node_data) = self._graph.node_weight(node_idx) {
                writeln!(writer, "{}", node_data)?;
            }
        }

        // Write edges
        for edge_idx in self._graph.edge_indices() {
            if let Some(edge) = self._graph.edge_weight(edge_idx) {
                if let Some((source_idx, sink_idx)) = self._graph.edge_endpoints(edge_idx) {
                    // E  e1  n1  n2  chr1,chr1,1700,2000,splice
                    writeln!(
                        writer,
                        "E\t{}\t{}\t{}\t{}",
                        edge.id, self._graph[source_idx].id, self._graph[sink_idx].id, edge.sv
                    )?;
                }
            }

            // Write groups
            for (group_id, group) in &self.groups {
                match group {
                    Group::Unordered { id, elements, .. } => {
                        let elements_str: Vec<String> = elements
                            .iter()
                            .map(|e| e.to_str().unwrap_or("").to_string())
                            .collect();
                        writeln!(
                            writer,
                            "U\t{}\t{}",
                            id.to_str().unwrap_or(""),
                            elements_str.join(" ")
                        )?;
                    }
                    Group::Ordered { id, elements, .. } => {
                        let elements_str: Vec<String> =
                            elements.iter().map(|e| e.to_string()).collect();
                        writeln!(
                            writer,
                            "O\t{}\t{}",
                            id.to_str().unwrap_or(""),
                            elements_str.join(" ")
                        )?;
                    }
                    Group::Chain { id, elements, .. } => {
                        // Skip writing chains that are duplicated with groups
                        if group_id != id {
                            continue;
                        }
                        let elements_str: Vec<String> = elements
                            .iter()
                            .map(|e| e.to_str().unwrap_or("").to_string())
                            .collect();
                        writeln!(
                            writer,
                            "C\t{}\t{}",
                            id.to_str().unwrap_or(""),
                            elements_str.join(" ")
                        )?;
                    }
                }
            }

            // Write attributes for nodes
            for node_idx in self._graph.node_indices() {
                if let Some(node) = self._graph.node_weight(node_idx) {
                    for (_tag, attr) in &node.attributes {
                        writeln!(writer, "A\tN\t{}\t{}", node.id, attr)?;
                    }
                }
            }

            // Write attributes for edges
            for edge_idx in self._graph.edge_indices() {
                if let Some(edge) = self._graph.edge_weight(edge_idx) {
                    for (_tag, attr) in &edge.attributes {
                        writeln!(writer, "A\tE\t{}\t{}", edge.id, attr)?;
                    }
                }
            }

            // Write attributes for groups
            for (id, group) in &self.groups {
                let group_type = match group {
                    Group::Unordered { .. } => "U",
                    Group::Ordered { .. } => "O",
                    Group::Chain { .. } => "C",
                };

                let attributes = match group {
                    Group::Unordered { attributes, .. } => attributes,
                    Group::Ordered { attributes, .. } => attributes,
                    Group::Chain { attributes, .. } => attributes,
                };

                for (_tag, attr) in attributes {
                    writeln!(
                        writer,
                        "A\t{}\t{}\t{}",
                        group_type,
                        id.to_str().unwrap_or(""),
                        attr
                    )?;
                }
            }

            writer.flush()?;
        }
        Ok(())
    }

    /// Helper method to find a node's ID by its index
    fn find_node_id_by_idx(&self, node_idx: NodeIndex) -> Option<&BString> {
        for (id, &idx) in &self.node_indices {
            if idx == node_idx {
                return Some(id);
            }
        }
        None
    }

    /// Get a node by its ID
    pub fn get_node(&self, id: &BStr) -> Option<&NodeData> {
        let node_idx = self.node_indices.get(id)?;
        self._graph.node_weight(*node_idx)
    }

    /// Get an edge by its ID
    pub fn get_edge(&self, id: &BStr) -> Option<&EdgeData> {
        let edge_idx = self.edge_indices.get(id)?;
        self._graph.edge_weight(*edge_idx)
    }

    /// Get all nodes in the graph
    pub fn get_nodes(&self) -> Vec<&NodeData> {
        self.node_indices
            .values()
            .filter_map(|&idx| self._graph.node_weight(idx))
            .collect()
    }

    /// Get all edges in the graph
    pub fn get_edges(&self) -> Vec<&EdgeData> {
        self.edge_indices
            .values()
            .filter_map(|&idx| self._graph.edge_weight(idx))
            .collect()
    }
}
