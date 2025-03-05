mod attr;
mod edge;
mod group;
mod header;
mod node;
mod path;

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::str::FromStr;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::{Context, Result, anyhow};
pub use attr::*;
use bstr::{BStr, BString, ByteSlice};
pub use edge::*;
pub use group::*;
pub use header::*;
pub use node::*;
pub use path::*;

use bon::Builder;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use rayon::prelude::*;
use serde_json::json;
use std::collections::VecDeque;
use tracing::debug;

/// The complete transcript segment graph
#[derive(Debug, Clone, Default, Builder)]
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
                return Ok(idx);
            }
            return Err(anyhow!("Node with ID {} not found in graph", id));
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

        // petgraph provide update_edge method to update edge data
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
            attributes: HashMap::new(),
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
            return Err(anyhow!("Group with ID {} already exists", id));
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
        let element_type = fields[1];
        let element_id: BString = fields[2].into();

        let attrs: Vec<Attribute> = fields
            .iter()
            .skip(3)
            .map(|s| s.parse())
            .collect::<Result<Vec<_>>>()
            .context("invalidate attribute line")?;

        match element_type {
            "N" => {
                if let Some(&node_idx) = self.node_indices.get(&element_id) {
                    if let Some(node_data) = self._graph.node_weight_mut(node_idx) {
                        for attr in attrs {
                            let tag = attr.tag.clone();
                            node_data.attributes.insert(tag, attr);
                        }
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
                        for attr in attrs {
                            let tag = attr.tag.clone();
                            edge_data.attributes.insert(tag, attr);
                        }
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
                        Group::Unordered { attributes, .. }
                        | Group::Ordered { attributes, .. }
                        | Group::Chain { attributes, .. } => {
                            for attr in attrs {
                                let tag = attr.tag.clone();
                                attributes.insert(tag, attr);
                            }
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
            for group in self.chains.clone().values() {
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
                        } else if i + 1 < elements.len() {
                            // Prevent index out of bounds
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

                                self.add_edge(source_id.as_bstr(), sink_id.as_bstr(), edge_data)?;
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
                    // ignore unknown record types
                    debug!("Ignoring unknown record type: {}", fields[0]);
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
                let mut nodes = Vec::with_capacity((elements.len() + 1) / 2);

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
                let mut edges = Vec::with_capacity(elements.len() / 2);

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
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "# Header")?;
        // Write headers
        for header in &self.headers {
            writeln!(writer, "{}", header)?;
        }

        let new_header = Header::builder().tag("PG").value("tsg").build();
        writeln!(writer, "{}", new_header)?;

        writeln!(writer, "# Nodes")?;
        // Write nodes
        for node_idx in self._graph.node_indices() {
            if let Some(node_data) = self._graph.node_weight(node_idx) {
                writeln!(writer, "{}", node_data)?;
            }
        }

        writeln!(writer, "# Edges")?;
        // Write edges
        for edge_ref in self._graph.edge_references() {
            let edge = edge_ref.weight();
            let source_idx = edge_ref.source();
            let sink_idx = edge_ref.target();

            // E  e1  n1  n2  chr1,chr1,1700,2000,splice
            if let (Some(source), Some(sink)) = (
                self._graph.node_weight(source_idx),
                self._graph.node_weight(sink_idx),
            ) {
                writeln!(
                    writer,
                    "E\t{}\t{}\t{}\t{}",
                    edge.id, source.id, sink.id, edge.sv
                )?;
            }
        }

        writeln!(writer, "# Groups")?;
        // Write groups
        let mut seen_chain_ids = HashSet::new();

        for group in self.groups.values() {
            match group {
                Group::Unordered { id, elements, .. } => {
                    let elements_str: Vec<String> = elements
                        .par_iter()
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
                        elements.par_iter().map(|e| e.to_string()).collect();
                    writeln!(
                        writer,
                        "O\t{}\t{}",
                        id.to_str().unwrap_or(""),
                        elements_str.join(" ")
                    )?;
                }
                Group::Chain { id, elements, .. } => {
                    // Skip writing chains that are duplicated with groups
                    if seen_chain_ids.contains(id) {
                        continue;
                    }
                    seen_chain_ids.insert(id);

                    let elements_str: Vec<String> = elements
                        .par_iter()
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

        writeln!(writer, "# Attributes")?;
        // Write attributes for nodes
        for node_idx in self._graph.node_indices() {
            if let Some(node) = self._graph.node_weight(node_idx) {
                for attr in node.attributes.values() {
                    writeln!(writer, "A\tN\t{}\t{}", node.id, attr)?;
                }
            }
        }

        // Write attributes for edges
        for edge_idx in self._graph.edge_indices() {
            if let Some(edge) = self._graph.edge_weight(edge_idx) {
                for attr in edge.attributes.values() {
                    writeln!(writer, "A\tE\t{}\t{}", edge.id, attr)?;
                }
            }
        }

        // Write attributes for groups
        for (id, group) in &self.groups {
            let (group_type, attributes) = match group {
                Group::Unordered { attributes, .. } => ("U", attributes),
                Group::Ordered { attributes, .. } => ("O", attributes),
                Group::Chain { attributes, .. } => ("C", attributes),
            };

            for attr in attributes.values() {
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
        Ok(())
    }

    /// Helper method to find a node's ID by its index
    pub fn find_node_id_by_idx(&self, node_idx: NodeIndex) -> Option<&BString> {
        self.node_indices
            .par_iter()
            .find_map_first(|(id, &idx)| if idx == node_idx { Some(id) } else { None })
    }

    pub fn get_node_by_idx(&self, node_idx: NodeIndex) -> Option<&NodeData> {
        self._graph.node_weight(node_idx)
    }

    /// Get a node by its ID
    pub fn get_node_by_id(&self, id: &str) -> Option<&NodeData> {
        let node_idx = self.node_indices.get(&BString::from(id))?;
        self._graph.node_weight(*node_idx)
    }

    /// Get an edge by its ID
    pub fn get_edge_by_id(&self, id: &str) -> Option<&EdgeData> {
        let edge_idx = self.edge_indices.get(&BString::from(id))?;
        self._graph.edge_weight(*edge_idx)
    }

    pub fn get_edge_by_idx(&self, edge_idx: EdgeIndex) -> Option<&EdgeData> {
        self._graph.edge_weight(edge_idx)
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

    /// Traverse the graph and return all valid paths from source nodes to sink nodes.
    ///
    /// A valid path must respect read continuity, especially for nodes with Intermediary (IN) reads.
    /// For nodes with IN reads, we ensure that:
    /// 1. The node shares at least one read with previous nodes in the path
    /// 2. The node can connect to at least one subsequent node that shares a read with it
    ///
    /// Example:
    /// n1 (r1) -> n3 (r1,r2) -> n4 (r1)
    /// n2 (r2) -> n3 (r1,r2) -> n5 (r2)
    ///
    /// If n3 has IN reads, then only these paths are valid:
    /// - n1 -> n3 -> n4 (valid because they all share read r1)
    /// - n2 -> n3 -> n5 (valid because they all share read r2)
    ///
    /// These paths would be invalid:
    /// - n1 -> n3 -> n5 (invalid because n1 and n5 don't share a common read)
    /// - n2 -> n3 -> n4 (invalid because n2 and n4 don't share a common read)
    pub fn traverse(&self) -> Result<Vec<TSGPath>> {
        // Find all source nodes (nodes with no incoming edges)
        let source_nodes: Vec<NodeIndex> = self
            ._graph
            .node_indices()
            .filter(|&idx| {
                self._graph
                    .edges_directed(idx, petgraph::Direction::Incoming)
                    .count()
                    == 0
            })
            .collect();

        if source_nodes.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_paths = Vec::new();
        let mut path_id = 0;

        // Cache node read IDs to avoid repeated lookups
        let mut node_read_ids_cache: HashMap<NodeIndex, HashSet<BString>> = HashMap::new();

        // Pre-compute node read IDs
        for node_idx in self._graph.node_indices() {
            if let Some(node) = self._graph.node_weight(node_idx) {
                let read_ids: HashSet<BString> =
                    node.reads.par_iter().map(|r| r.id.clone()).collect();
                node_read_ids_cache.insert(node_idx, read_ids);
            }
        }

        // For each source node, perform a traversal
        for &start_node in &source_nodes {
            // Skip nodes with no reads
            if let Some(read_set) = node_read_ids_cache.get(&start_node) {
                if read_set.is_empty() {
                    continue;
                }

                let mut queue = VecDeque::new();
                // (node, path_so_far, active_reads)
                let mut initial_path = TSGPath::builder().graph(self).build();
                initial_path.add_node(start_node);

                queue.push_back((start_node, initial_path, read_set.clone()));

                while let Some((current_node, mut path, active_reads)) = queue.pop_front() {
                    // Get outgoing edges
                    let outgoing_edges: Vec<_> = self
                        ._graph
                        .edges_directed(current_node, petgraph::Direction::Outgoing)
                        .collect();

                    // If this is a sink node (no outgoing edges), save the path
                    if outgoing_edges.is_empty() {
                        path.set_id(format!("{}", path_id).as_str());
                        path.validate()?;
                        all_paths.push(path);
                        path_id += 1;
                        continue;
                    }

                    for edge_ref in outgoing_edges {
                        let edge_idx = edge_ref.id();
                        let target_node = edge_ref.target();

                        if let Some(target_read_ids) = node_read_ids_cache.get(&target_node) {
                            // Calculate reads that continue from current path to target
                            let continuing_reads: HashSet<_> = active_reads
                                .par_iter()
                                .filter(|id| target_read_ids.contains(*id))
                                .cloned()
                                .collect();

                            if continuing_reads.is_empty() {
                                // No read continuity, skip this edge
                                continue;
                            }

                            // Check if target node has IN reads
                            let has_in_reads =
                                if let Some(target_data) = self._graph.node_weight(target_node) {
                                    target_data
                                        .reads
                                        .par_iter()
                                        .any(|r| r.identity == ReadIdentity::IN)
                                } else {
                                    false
                                };

                            if has_in_reads {
                                // For IN nodes, check if there's a valid path forward
                                let mut can_continue = false;
                                let outgoing_from_target: Vec<_> = self
                                    ._graph
                                    .edges_directed(target_node, petgraph::Direction::Outgoing)
                                    .map(|e| e.target())
                                    .collect();

                                for &next_node in &outgoing_from_target {
                                    if let Some(next_read_ids) = node_read_ids_cache.get(&next_node)
                                    {
                                        // Check if there's at least one read that continues through
                                        if continuing_reads
                                            .par_iter()
                                            .any(|id| next_read_ids.contains(id))
                                        {
                                            can_continue = true;
                                            break;
                                        }
                                    }
                                }

                                if !can_continue && !outgoing_from_target.is_empty() {
                                    // Has outgoing edges but no valid continuation, skip this edge
                                    continue;
                                }
                            }

                            // Create new path and continue traversal
                            let mut new_path = path.clone();
                            new_path.add_edge(edge_idx);
                            new_path.add_node(target_node);
                            queue.push_back((target_node, new_path, continuing_reads));
                        }
                    }
                }
            }
        }

        Ok(all_paths)
    }

    pub fn to_dot(&self, node_label: bool, edge_label: bool) -> Result<String> {
        let mut config = vec![];
        if node_label {
            config.push(Config::NodeIndexLabel);
        }
        if edge_label {
            config.push(Config::EdgeIndexLabel);
        }

        let dot = Dot::with_config(&self._graph, &config);
        Ok(format!("{:?}", dot))
    }
    pub fn to_json(&self) -> Result<serde_json::Value> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Process all nodes
        for node_idx in self._graph.node_indices() {
            if let Some(node) = self._graph.node_weight(node_idx) {
                if let Ok(node_json) = node.to_json(None) {
                    nodes.push(node_json);
                }
            }
        }

        // Process all edges
        for edge_idx in self._graph.edge_indices() {
            if let Some(edge) = self._graph.edge_weight(edge_idx) {
                let edge_endpoints = self._graph.edge_endpoints(edge_idx);

                if let Some((source, target)) = edge_endpoints {
                    let source_id = self.find_node_id_by_idx(source);
                    let target_id = self.find_node_id_by_idx(target);

                    if let (Some(source_id), Some(target_id)) = (source_id, target_id) {
                        let edge_data = json!({
                            "data": {
                                "id": edge.id.to_str().unwrap(),
                                "source": source_id.to_str().unwrap(),
                                "target": target_id.to_str().unwrap(),
                                "breakpoints": format!("{}", edge.sv)
                            }
                        });
                        edges.push(edge_data);
                    }
                }
            }
        }

        // Combine nodes and edges into a Cytoscape-compatible format
        let elements = json!({
            "directed": true,
            "multigraph": true,
            "elements": {
            "nodes": nodes,
            "edges": edges
            }
        });

        Ok(elements)
    }

    pub fn annotate_node_with_sequence<P: AsRef<Path>>(
        &mut self,
        reference_genome_path: P,
    ) -> Result<()> {
        use noodles::fasta;
        let mut reader = fasta::io::indexed_reader::Builder::default()
            .build_from_path(reference_genome_path.as_ref())?;

        for node_idx in self.node_indices.values() {
            let node_data = self._graph.node_weight_mut(*node_idx).unwrap();

            let region = format!(
                "{}:{}-{}",
                node_data.reference_id,
                node_data.reference_start() - 1, // 0-based to 1-based
                node_data.reference_end(),
            )
            .parse()?;
            let record = reader.query(&region)?;
            node_data.sequence = Some(record.sequence().as_ref().into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_graph() {
        let graph = TSGraph::new();
        assert_eq!(graph.headers.len(), 0);
        assert_eq!(graph.get_nodes().len(), 0);
        assert_eq!(graph.get_edges().len(), 0);
        assert_eq!(graph.groups.len(), 0);
        assert_eq!(graph.chains.len(), 0);
    }

    #[test]
    fn test_add_node() -> Result<()> {
        let mut graph = TSGraph::new();
        let node = NodeData {
            id: "node1".into(),
            reference_id: "chr1".into(),
            ..Default::default()
        };

        graph.add_node(node.clone())?;

        assert_eq!(graph.get_nodes().len(), 1);
        assert_eq!(graph.get_node_by_id("node1").unwrap().id, node.id);

        Ok(())
    }

    #[test]
    fn test_add_edge() -> Result<()> {
        let mut graph = TSGraph::new();

        // Add nodes first
        let node1 = NodeData {
            id: "node1".into(),
            reference_id: "chr1".into(),
            ..Default::default()
        };

        let node2 = NodeData {
            id: "node2".into(),
            ..Default::default()
        };

        graph.add_node(node1)?;
        graph.add_node(node2)?;

        // Add edge
        let edge = EdgeData {
            id: "edge1".into(),
            ..Default::default()
        };

        graph.add_edge("node1".into(), "node2".into(), edge.clone())?;

        assert_eq!(graph.get_edges().len(), 1);
        assert_eq!(graph.get_edge_by_id("edge1").unwrap().id, edge.id);

        Ok(())
    }

    #[test]
    fn test_parse_header_line() -> Result<()> {
        let mut graph = TSGraph::new();
        let fields = vec!["H", "VN", "1.0"];

        graph.parse_header_line(&fields)?;

        assert_eq!(graph.headers.len(), 1);
        assert_eq!(graph.headers[0].tag, "VN");
        assert_eq!(graph.headers[0].value, "1.0");

        Ok(())
    }

    #[test]
    fn test_parse_node_line() -> Result<()> {
        let mut graph = TSGraph::new();
        let line = "N\tnode1\tchr1:+:100-200\tread1:SO,read2:IN\tACGT";

        graph.parse_node_line(line)?;

        let node = graph.get_node_by_id("node1").unwrap();
        assert_eq!(node.id, "node1");
        assert_eq!(node.exons.exons.len(), 1);
        assert_eq!(node.exons.exons[0].start, 100);
        assert_eq!(node.exons.exons[0].end, 200);
        assert_eq!(
            node.reads,
            vec![
                ReadData::builder().id("read1").identity("SO").build(),
                ReadData::builder().id("read2").identity("IN").build(),
            ]
        );
        assert_eq!(node.sequence, Some("ACGT".into()));

        Ok(())
    }

    #[test]
    fn test_parse_unordered_group() -> Result<()> {
        let mut graph = TSGraph::new();
        let fields = vec!["U", "group1", "node1", "node2", "edge1"];

        graph.parse_unordered_group_line(&fields)?;

        assert_eq!(graph.groups.len(), 1);
        if let Group::Unordered { id, elements, .. } = &graph.groups["group1".as_bytes()] {
            assert_eq!(id, "group1");
            assert_eq!(elements.len(), 3);
            assert_eq!(elements[0], "node1");
            assert_eq!(elements[1], "node2");
            assert_eq!(elements[2], "edge1");
        } else {
            panic!("Expected Unordered group");
        }

        Ok(())
    }

    #[test]
    fn test_read_from_file() -> Result<()> {
        let file = "tests/data/test.tsg";
        let graph = TSGraph::from_file(file)?;

        assert_eq!(graph.headers.len(), 2);
        assert_eq!(graph.get_nodes().len(), 5);
        assert_eq!(graph.get_edges().len(), 4);

        graph.write_to_file("tests/data/test_write.tsg")?;

        Ok(())
    }

    #[test]
    fn test_traverse() -> Result<()> {
        let file = "tests/data/test.tsg";
        let graph = TSGraph::from_file(file)?;

        let paths = graph.traverse()?;
        // assert_eq!(paths.len(), 2);

        for path in paths {
            println!("{}", path);
        }
        Ok(())
    }

    #[test]
    fn test_to_dot() -> Result<()> {
        let file = "tests/data/test.tsg";
        let graph = TSGraph::from_file(file)?;

        let dot = graph.to_dot(true, true)?;
        println!("{}", dot);

        Ok(())
    }

    #[test]
    fn test_to_json() -> Result<()> {
        let file = "tests/data/test.tsg";
        let graph = TSGraph::from_file(file)?;

        let json = graph.to_json()?;
        println!("{}", json);
        Ok(())
    }
}
