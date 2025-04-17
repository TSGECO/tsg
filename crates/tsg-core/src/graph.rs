mod analysis;
mod attr;
mod edge;
mod group;
mod header;
mod node;
mod path;
mod utils;

use noodles::fasta;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;
use std::str::FromStr;
use tracing::debug;
use tracing::warn;

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::{Context, Result, anyhow};
use bstr::{BStr, BString, ByteSlice};

pub use analysis::*;
pub use attr::*;
pub use edge::*;
pub use group::*;
pub use header::*;
pub use node::*;
pub use path::*;
pub use utils::*;

use bon::Builder;
use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use rayon::prelude::*;
use serde_json::json;
use std::collections::VecDeque;

pub const DEFAULT_GRAPH_ID: &str = "G.graph";
/// Represents a graph section within the TSG file
#[derive(Debug, Clone, Default, Builder)]
pub struct GraphSection {
    pub id: BString,
    pub attributes: HashMap<BString, Attribute>,
    _graph: DiGraph<NodeData, EdgeData>,
    pub node_indices: HashMap<BString, NodeIndex>,
    pub edge_indices: HashMap<BString, EdgeIndex>,
    pub groups: HashMap<BString, Group>,
    pub chains: HashMap<BString, Group>,
}

impl GraphSection {
    /// Create a new empty GraphSection
    pub fn new(id: BString) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn new_default_graph() -> Self {
        Self::new(DEFAULT_GRAPH_ID.into())
    }

    pub fn node_indices_to_ids(&self) -> HashMap<NodeIndex, BString> {
        self.node_indices
            .par_iter()
            .map(|(id, &idx)| (idx, id.clone()))
            .collect()
    }

    pub fn edge_indices_to_ids(&self) -> HashMap<EdgeIndex, BString> {
        self.edge_indices
            .par_iter()
            .map(|(id, &idx)| (idx, id.clone()))
            .collect()
    }

    pub fn node_weight(&self, node_idx: NodeIndex) -> Option<&NodeData> {
        self._graph.node_weight(node_idx)
    }

    pub fn edge_weight(&self, edge_idx: EdgeIndex) -> Option<&EdgeData> {
        self._graph.edge_weight(edge_idx)
    }

    pub fn in_degree(&self, node_idx: NodeIndex) -> usize {
        self._graph
            .edges_directed(node_idx, petgraph::Direction::Incoming)
            .count()
    }

    pub fn out_degree(&self, node_idx: NodeIndex) -> usize {
        self._graph
            .edges_directed(node_idx, petgraph::Direction::Outgoing)
            .count()
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

        debug!(
            "graph {} add node {} ;len of node_indices: {}",
            self.id,
            id,
            self.node_indices.len() + 1
        );

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

        // Get source node index or create it if it doesn't exist
        let source_idx = match self.node_indices.get(source_id) {
            Some(&idx) => idx,
            None => {
                // Create a placeholder node if it doesn't exist
                let placeholder_node = NodeData {
                    id: source_id.to_owned(),
                    ..Default::default()
                };
                self.add_node(placeholder_node)?
            }
        };

        // Get sink node index or create it if it doesn't exist
        let sink_idx = match self.node_indices.get(sink_id) {
            Some(&idx) => idx,
            None => {
                // Create a placeholder node if it doesn't exist
                let placeholder_node = NodeData {
                    id: sink_id.to_owned(),
                    ..Default::default()
                };
                self.add_node(placeholder_node)?
            }
        };

        // petgraph provide update_edge method to update edge data
        let edge_idx = self._graph.update_edge(source_idx, sink_idx, edge_data);

        self.edge_indices.insert(id, edge_idx);
        Ok(edge_idx)
    }

    // Methods from old TSGraph that should now belong to GraphSection

    /// Build graph based on the current state
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

        if self.id == DEFAULT_GRAPH_ID {
            // ignore default graph
            return Ok(());
        }

        if self.node_indices.is_empty() || self.edge_indices.is_empty() {
            warn!(
                "Graph {} has no nodes/edges defined and no chains available",
                self.id
            );
        }

        Ok(())
    }

    // Additional GraphSection methods...
    pub fn node_by_idx(&self, node_idx: NodeIndex) -> Option<&NodeData> {
        self._graph.node_weight(node_idx)
    }

    pub fn node_by_id(&self, id: &str) -> Option<&NodeData> {
        let node_idx = self.node_indices.get(&BString::from(id))?;
        self._graph.node_weight(*node_idx)
    }

    pub fn edge_by_id(&self, id: &str) -> Option<&EdgeData> {
        let edge_idx = self.edge_indices.get(&BString::from(id))?;
        self._graph.edge_weight(*edge_idx)
    }

    pub fn edge_by_idx(&self, edge_idx: EdgeIndex) -> Option<&EdgeData> {
        self._graph.edge_weight(edge_idx)
    }

    pub fn nodes(&self) -> Vec<&NodeData> {
        self.node_indices
            .values()
            .map(|&idx| self._graph.node_weight(idx).unwrap())
            .collect()
    }

    pub fn edges(&self) -> Vec<&EdgeData> {
        self.edge_indices
            .values()
            .map(|&idx| self._graph.edge_weight(idx).unwrap())
            .collect()
    }

    /// Helper method to find a node's ID by its index
    pub fn find_node_id_by_idx(&self, node_idx: NodeIndex) -> Option<&BString> {
        self.node_indices
            .par_iter()
            .find_map_first(|(id, &idx)| if idx == node_idx { Some(id) } else { None })
    }

    // Other methods from TSGraph that make sense at the graph section level

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

                while let Some((current_node, path, active_reads)) = queue.pop_front() {
                    // Get outgoing edges
                    let outgoing_edges: Vec<_> = self
                        ._graph
                        .edges_directed(current_node, petgraph::Direction::Outgoing)
                        .collect();

                    // If this is a sink node (no outgoing edges), save the path
                    if outgoing_edges.is_empty() {
                        path.validate()?;
                        all_paths.push(path);
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

                    // get reads from source node and target node
                    // the weight will be the intersection of reads
                    let source_data = self.node_by_idx(source).unwrap();
                    let target_data = self.node_by_idx(target).unwrap();

                    // get the intersection of reads
                    let source_reads = source_data
                        .reads
                        .iter()
                        .map(|r| r.id.clone())
                        .collect::<HashSet<_>>();
                    let target_reads = target_data
                        .reads
                        .iter()
                        .map(|r| r.id.clone())
                        .collect::<HashSet<_>>();
                    let edge_weight = source_reads
                        .intersection(&target_reads)
                        .collect::<HashSet<_>>()
                        .len();

                    if let (Some(source_id), Some(target_id)) = (source_id, target_id) {
                        let edge_data = json!({
                            "data": {
                                "id": edge.id.to_str().unwrap(),
                                "source": source_id.to_str().unwrap(),
                                "target": target_id.to_str().unwrap(),
                                "weight": edge_weight,
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

/// Represents a link between elements in different graphs
#[derive(Debug, Clone, Default, Builder)]
pub struct InterGraphLink {
    pub id: BString,
    pub source_graph: BString,
    pub source_element: BString,
    pub target_graph: BString,
    pub target_element: BString,
    pub link_type: BString,
    #[builder(default)]
    pub attributes: HashMap<BString, Attribute>,
}

/// The complete transcript segment graph containing multiple graph sections
#[derive(Debug, Clone, Default, Builder)]
pub struct TSGraph {
    pub headers: Vec<Header>,
    pub graphs: HashMap<BString, GraphSection>,
    pub links: Vec<InterGraphLink>,
    current_graph_id: Option<BString>, // Tracks which graph is currently active during parsing
}

impl TSGraph {
    /// Create a new empty TSGraph
    pub fn new() -> Self {
        let graph = GraphSection::new_default_graph();
        let mut graphs = HashMap::new();
        graphs.insert(graph.id.clone(), graph);
        Self {
            graphs,
            current_graph_id: Some(DEFAULT_GRAPH_ID.into()),
            ..Default::default()
        }
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

    /// Parse a graph section line
    fn parse_graph_line(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() < 2 {
            return Err(anyhow!("Invalid graph line format"));
        }

        let graph_id: BString = fields[1].into();

        // Check if graph with this ID already exists
        if self.graphs.contains_key(&graph_id) {
            return Err(anyhow!(
                "Graph with ID {} already exists",
                graph_id.to_str().unwrap_or("")
            ));
        }

        // Create new graph section
        let mut graph_section = GraphSection::new(graph_id.clone());

        // Parse optional attributes
        if fields.len() > 2 {
            for attr_str in &fields[2..] {
                let attr = attr_str.parse::<Attribute>()?;
                graph_section.attributes.insert(attr.tag.clone(), attr);
            }
        }

        // Update current graph ID and add to graphs map
        self.current_graph_id = Some(graph_id.clone());
        self.graphs.insert(graph_id, graph_section);

        Ok(())
    }

    /// Get the current graph section (or error if none is active)
    fn current_graph_mut(&mut self) -> Result<&mut GraphSection> {
        if let Some(graph_id) = &self.current_graph_id {
            if let Some(graph) = self.graphs.get_mut(graph_id) {
                return Ok(graph);
            }
        }
        Err(anyhow!("No active graph section"))
    }

    /// Parse an inter-graph link line
    fn parse_link_line(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() < 5 {
            return Err(anyhow!("Invalid link line format"));
        }

        let id: BString = fields[1].into();

        // Parse source and target references (format: graph_id:element_id)
        let source_ref: Vec<&str> = fields[2].splitn(2, ':').collect();
        let target_ref: Vec<&str> = fields[3].splitn(2, ':').collect();

        if source_ref.len() != 2 || target_ref.len() != 2 {
            return Err(anyhow!("Invalid element reference format in link line"));
        }

        let source_graph: BString = source_ref[0].into();
        let source_element: BString = source_ref[1].into();
        let target_graph: BString = target_ref[0].into();
        let target_element: BString = target_ref[1].into();

        // Verify the referenced graphs exist
        if !self.graphs.contains_key(&source_graph) {
            return Err(anyhow!(
                "Source graph {} not found",
                source_graph.to_str().unwrap_or("")
            ));
        }
        if !self.graphs.contains_key(&target_graph) {
            return Err(anyhow!(
                "Target graph {} not found",
                target_graph.to_str().unwrap_or("")
            ));
        }

        let link_type: BString = fields[4].into();

        let mut link = InterGraphLink::builder()
            .id(id)
            .source_graph(source_graph)
            .source_element(source_element)
            .target_graph(target_graph)
            .target_element(target_element)
            .link_type(link_type)
            .build();

        // Parse optional attributes
        if fields.len() > 5 {
            for attr_str in &fields[5..] {
                let attr = attr_str.parse::<Attribute>()?;
                link.attributes.insert(attr.tag.clone(), attr);
            }
        }

        self.links.push(link);
        Ok(())
    }

    /// Parse a node line
    fn parse_node_line(&mut self, fields: &str) -> Result<()> {
        let node_data = NodeData::from_str(fields)?;
        let graph = self.current_graph_mut()?;
        graph.add_node(node_data)?;
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

        let edge_data = EdgeData::builder().id(id).sv(sv).build();

        let graph = self.current_graph_mut()?;
        graph.add_edge(source_id.as_bstr(), sink_id.as_bstr(), edge_data)?;
        Ok(())
    }

    /// Parse an unordered group line
    fn parse_unordered_group_line(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() < 3 {
            return Err(anyhow!("Invalid unordered group line format"));
        }

        let id: BString = fields[1].into();
        let graph = self.current_graph_mut()?;

        // Check for duplicate group name
        if graph.groups.contains_key(&id) {
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

        graph.groups.insert(id, group);
        Ok(())
    }

    /// Parse a path line
    fn parse_path_line(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() < 3 {
            return Err(anyhow!("Invalid path line format"));
        }

        let id: BString = fields[1].into();
        let graph = self.current_graph_mut()?;

        // Check for duplicate group name
        if graph.groups.contains_key(&id) {
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

        graph.groups.insert(id, group);
        Ok(())
    }

    /// Parse a chain line
    fn parse_chain_line(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() < 3 {
            return Err(anyhow!("Invalid chain line format"));
        }

        let id: BString = fields[1].into();
        let graph = self.current_graph_mut()?;

        // Check for duplicate group name
        if graph.groups.contains_key(&id) {
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
        graph.chains.insert(id.clone(), group.clone());
        graph.groups.insert(id, group);
        Ok(())
    }

    /// Parse an attribute line
    fn parse_attribute_line(&mut self, fields: &[&str]) -> Result<()> {
        if fields.len() < 4 {
            return Err(anyhow!("Invalid attribute line format"));
        }

        let element_type = fields[1];
        let element_id: BString = fields[2].into();
        let graph = self.current_graph_mut()?;

        let attrs: Vec<Attribute> = fields
            .iter()
            .skip(3)
            .map(|s| s.parse())
            .collect::<Result<Vec<_>>>()
            .context("invalidate attribute line")?;

        match element_type {
            "N" => {
                if let Some(&node_idx) = graph.node_indices.get(&element_id) {
                    if let Some(node_data) = graph._graph.node_weight_mut(node_idx) {
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
                if let Some(&edge_idx) = graph.edge_indices.get(&element_id) {
                    if let Some(edge_data) = graph._graph.edge_weight_mut(edge_idx) {
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
            "U" | "P" | "C" => {
                if let Some(group) = graph.groups.get_mut(&element_id) {
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
            "G" => {
                // Handle graph attributes
                if let Some(graph_section) = self.graphs.get_mut(&element_id) {
                    for attr in attrs {
                        let tag = attr.tag.clone();
                        graph_section.attributes.insert(tag, attr);
                    }
                } else {
                    return Err(anyhow!("Graph with ID {} not found", element_id));
                }
            }
            _ => {
                return Err(anyhow!("Unknown element type: {}", element_type));
            }
        }

        Ok(())
    }

    /// Validate all graphs and their paths
    fn validate(&self) -> Result<()> {
        // Validate each graph section
        for (graph_id, graph) in &self.graphs {
            // Validate paths against the graph
            for (id, group) in &graph.groups {
                if let Group::Ordered { elements, .. } = group {
                    // Validate that all elements in the path exist in the graph
                    for element in elements {
                        let element_exists = graph.node_indices.contains_key(&element.id)
                            || graph.edge_indices.contains_key(&element.id)
                            || graph.groups.contains_key(&element.id);

                        if !element_exists {
                            return Err(anyhow!(
                                "Path {} in graph {} references non-existent element {}",
                                id,
                                graph_id,
                                element.id
                            ));
                        }
                    }
                }
            }
        }

        // Validate all inter-graph links
        for link in &self.links {
            // Check source element exists
            let source_graph = self.graphs.get(&link.source_graph).ok_or_else(|| {
                anyhow!(
                    "Link {} references non-existent graph {}",
                    link.id,
                    link.source_graph
                )
            })?;

            let source_exists = source_graph.node_indices.contains_key(&link.source_element)
                || source_graph.edge_indices.contains_key(&link.source_element)
                || source_graph.groups.contains_key(&link.source_element);

            if !source_exists {
                return Err(anyhow!(
                    "Link {} references non-existent element {}:{}",
                    link.id,
                    link.source_graph,
                    link.source_element
                ));
            }

            // Check target element exists
            let target_graph = self.graphs.get(&link.target_graph).ok_or_else(|| {
                anyhow!(
                    "Link {} references non-existent graph {}",
                    link.id,
                    link.target_graph
                )
            })?;

            let target_exists = target_graph.node_indices.contains_key(&link.target_element)
                || target_graph.edge_indices.contains_key(&link.target_element)
                || target_graph.groups.contains_key(&link.target_element);

            if !target_exists {
                return Err(anyhow!(
                    "Link {} references non-existent element {}:{}",
                    link.id,
                    link.target_graph,
                    link.target_element
                ));
            }
        }

        Ok(())
    }

    pub fn from_reader<R: BufRead>(reader: R) -> Result<Self> {
        let mut tsgraph = TSGraph::new();

        // Create a default graph if needed for backward compatibility
        let default_graph_id: BString = DEFAULT_GRAPH_ID.into();
        let default_graph = GraphSection::new(default_graph_id.clone());
        tsgraph
            .graphs
            .insert(default_graph_id.clone(), default_graph);

        tsgraph.current_graph_id = Some(default_graph_id);

        // First pass: Parse all record types
        for line in reader.lines() {
            let line = line?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.is_empty() {
                continue;
            }

            match fields[0] {
                "H" => tsgraph.parse_header_line(&fields)?,
                "G" => tsgraph.parse_graph_line(&fields)?,
                "N" => tsgraph.parse_node_line(&line)?,
                "E" => tsgraph.parse_edge_line(&fields)?,
                "U" => tsgraph.parse_unordered_group_line(&fields)?,
                "P" => tsgraph.parse_path_line(&fields)?,
                "C" => tsgraph.parse_chain_line(&fields)?,
                "A" => tsgraph.parse_attribute_line(&fields)?,
                "L" => tsgraph.parse_link_line(&fields)?,
                _ => {
                    // ignore unknown record types
                    debug!("Ignoring unknown record type: {}", fields[0]);
                }
            }
        }

        // Second pass: Ensure all graphs are built and validate
        for graph_section in tsgraph.graphs.values_mut() {
            // Populate chains hash map from groups if needed
            for (id, group) in &graph_section.groups {
                if let Group::Chain { .. } = group {
                    if !graph_section.chains.contains_key(id) {
                        graph_section.chains.insert(id.clone(), group.clone());
                    }
                }
            }

            // Ensure graph is built
            graph_section.ensure_graph_is_built()?;
        }

        // Validate all graphs and links
        tsgraph.validate()?;

        // pop the default graph if it's empty
        if let Some(default_graph) = tsgraph.graph(DEFAULT_GRAPH_ID) {
            if default_graph.node_indices.is_empty() {
                tsgraph.graphs.remove(&BString::from(DEFAULT_GRAPH_ID));
            }
        }
        Ok(tsgraph)
    }

    /// Parse a TSG file and construct a TSGraph
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::from_reader(reader)
    }

    /// Write the TSGraph to writer
    pub fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        // Write global headers
        writeln!(writer, "# Global header")?;
        for header in &self.headers {
            writeln!(writer, "{}", header)?;
        }

        let new_header = Header::builder().tag("PG").value("tsg").build();
        writeln!(writer, "{}", new_header)?;

        // Write each graph section
        for (graph_id, graph) in &self.graphs {
            // Skip writing the default graph header if it's empty
            if graph_id == &BString::from(DEFAULT_GRAPH_ID) && graph.node_indices.is_empty() {
                continue;
            }

            writeln!(writer, "\n# Graph: {}", graph_id)?;

            // Write graph section header
            write!(writer, "G\t{}", graph_id)?;
            for attr in graph.attributes.values() {
                write!(writer, "\t{}", attr)?;
            }
            writeln!(writer)?;

            // Write nodes
            writeln!(writer, "# Nodes")?;
            for node_idx in graph._graph.node_indices() {
                if let Some(node_data) = graph._graph.node_weight(node_idx) {
                    writeln!(writer, "{}", node_data)?;
                }
            }

            // Write edges
            writeln!(writer, "# Edges")?;
            for edge_ref in graph._graph.edge_references() {
                let edge = edge_ref.weight();
                let source_idx = edge_ref.source();
                let sink_idx = edge_ref.target();

                if let (Some(source), Some(sink)) = (
                    graph._graph.node_weight(source_idx),
                    graph._graph.node_weight(sink_idx),
                ) {
                    writeln!(
                        writer,
                        "E\t{}\t{}\t{}\t{}",
                        edge.id, source.id, sink.id, edge.sv
                    )?;
                }
            }

            // Write groups
            writeln!(writer, "# Groups")?;
            let mut seen_chain_ids = HashSet::new();

            for group in graph.groups.values() {
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
                            "P\t{}\t{}",
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

            // Write attributes for this graph section
            writeln!(writer, "# Attributes")?;

            // Write attributes for nodes
            for node_idx in graph._graph.node_indices() {
                if let Some(node) = graph._graph.node_weight(node_idx) {
                    for attr in node.attributes.values() {
                        writeln!(writer, "A\tN\t{}\t{}", node.id, attr)?;
                    }
                }
            }

            // Write attributes for edges
            for edge_idx in graph._graph.edge_indices() {
                if let Some(edge) = graph._graph.edge_weight(edge_idx) {
                    for attr in edge.attributes.values() {
                        writeln!(writer, "A\tE\t{}\t{}", edge.id, attr)?;
                    }
                }
            }

            // Write attributes for groups
            for (id, group) in &graph.groups {
                let (group_type, attributes) = match group {
                    Group::Unordered { attributes, .. } => ("U", attributes),
                    Group::Ordered { attributes, .. } => ("P", attributes),
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
        }

        // Write inter-graph links
        if !self.links.is_empty() {
            writeln!(writer, "\n# Inter-graph links")?;
            for link in &self.links {
                write!(
                    writer,
                    "L\t{}\t{}:{}\t{}:{}\t{}",
                    link.id,
                    link.source_graph,
                    link.source_element,
                    link.target_graph,
                    link.target_element,
                    link.link_type
                )?;

                for attr in link.attributes.values() {
                    write!(writer, "\t{}", attr)?;
                }
                writeln!(writer)?;
            }
        }

        writer.flush()?;
        Ok(())
    }

    /// Write the TSGraph to a TSG file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.to_writer(&mut writer)
    }

    // Helper methods for accessing graph elements

    /// Get a graph section by its ID
    pub fn graph(&self, id: &str) -> Option<&GraphSection> {
        self.graphs.get(&BString::from(id))
    }

    pub fn graph_mut(&mut self, id: &str) -> Option<&mut GraphSection> {
        self.graphs.get_mut(&BString::from(id))
    }

    /// get default graph
    pub fn default_graph(&self) -> Option<&GraphSection> {
        self.graphs.get(&BString::from(DEFAULT_GRAPH_ID))
    }

    pub fn default_graph_mut(&mut self) -> Option<&mut GraphSection> {
        self.graphs.get_mut(&BString::from(DEFAULT_GRAPH_ID))
    }

    /// Get a node by its ID and graph ID
    pub fn node(&self, graph_id: &str, node_id: &str) -> Option<&NodeData> {
        let graph = self.graphs.get(&BString::from(graph_id))?;
        graph.node_by_id(node_id)
    }

    /// Get an edge by its ID and graph ID
    pub fn edge(&self, graph_id: &str, edge_id: &str) -> Option<&EdgeData> {
        let graph = self.graphs.get(&BString::from(graph_id))?;
        graph.edge_by_id(edge_id)
    }

    /// Get the nodes in a chain in order
    pub fn chain_nodes(&self, graph_id: &str, chain_id: &BStr) -> Option<Vec<NodeIndex>> {
        let graph = self.graphs.get(&BString::from(graph_id))?;
        let group = graph.chains.get(chain_id)?;

        match group {
            Group::Chain { elements, .. } => {
                let mut nodes = Vec::with_capacity(elements.len().div_ceil(2));

                for (i, element_id) in elements.iter().enumerate() {
                    if i % 2 == 0 {
                        // Nodes are at even positions (0, 2, 4...)
                        if let Some(&node_idx) = graph.node_indices.get(element_id) {
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
    pub fn chain_edges(&self, graph_id: &str, chain_id: &BStr) -> Option<Vec<EdgeIndex>> {
        let graph = self.graphs.get(&BString::from(graph_id))?;
        let group = graph.chains.get(chain_id)?;

        match group {
            Group::Chain { elements, .. } => {
                let mut edges = Vec::with_capacity(elements.len() / 2);

                for (i, element_id) in elements.iter().enumerate() {
                    if i % 2 == 1 {
                        // Edges are at odd positions (1, 3, 5...)
                        if let Some(&edge_idx) = graph.edge_indices.get(element_id) {
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

    /// Helper method to find a node's ID by its index
    pub fn find_node_id_by_idx(&self, graph_id: &str, node_idx: NodeIndex) -> Option<&BString> {
        let graph = self.graphs.get(&BString::from(graph_id))?;
        graph
            .node_indices
            .par_iter()
            .find_map_first(|(id, &idx)| if idx == node_idx { Some(id) } else { None })
    }

    pub fn node_by_idx(&self, graph_id: &str, node_idx: NodeIndex) -> Option<&NodeData> {
        let graph = self.graphs.get(&BString::from(graph_id))?;
        graph.node_by_idx(node_idx)
    }

    pub fn edge_by_idx(&self, graph_id: &str, edge_idx: EdgeIndex) -> Option<&EdgeData> {
        let graph = self.graphs.get(&BString::from(graph_id))?;
        graph.edge_by_idx(edge_idx)
    }

    /// Get all nodes in the graph
    pub fn nodes(&self, graph_id: &str) -> Vec<&NodeData> {
        let graph = self.graphs.get(&BString::from(graph_id)).unwrap();
        graph
            .node_indices
            .values()
            .filter_map(|&idx| graph._graph.node_weight(idx))
            .collect()
    }

    /// Get all edges in the graph
    pub fn edges(&self, graph_id: &str) -> Vec<&EdgeData> {
        let graph = self.graphs.get(&BString::from(graph_id)).unwrap();
        graph
            .edge_indices
            .values()
            .filter_map(|&idx| graph._graph.edge_weight(idx))
            .collect()
    }

    /// Traverse the graph and return all valid paths from source nodes to sink nodes.
    pub fn traverse_by_id(&self, graph_id: &str) -> Result<Vec<TSGPath>> {
        let graph = self.graphs.get(&BString::from(graph_id)).unwrap();
        graph.traverse()
    }

    /// traverse all graphs
    pub fn traverse_all_graphs(&self) -> Result<Vec<TSGPath>> {
        let all_paths = self
            .graphs
            .values()
            .try_fold(Vec::new(), |mut all_paths, graph| {
                let paths = graph.traverse()?;
                all_paths.extend(paths);
                Ok(all_paths)
            });
        all_paths
    }

    pub fn to_dot_by_id(
        &self,
        graph_id: &str,
        node_label: bool,
        edge_label: bool,
    ) -> Result<String> {
        let graph = self.graphs.get(&BString::from(graph_id)).unwrap();
        graph.to_dot(node_label, edge_label)
    }

    pub fn to_json_by_id(&self, graph_id: &str) -> Result<serde_json::Value> {
        let graph = self.graphs.get(&BString::from(graph_id)).unwrap();
        graph.to_json()
    }
}

impl FromStr for TSGraph {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        let reader = BufReader::new(s.as_bytes());
        Self::from_reader(reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_graph() {
        let graph = TSGraph::new();
        assert_eq!(graph.headers.len(), 0);
        assert_eq!(graph.default_graph().unwrap().nodes().len(), 0);
        assert_eq!(graph.edges(DEFAULT_GRAPH_ID).len(), 0);
        assert_eq!(graph.graphs.len(), 1);
        assert_eq!(graph.links.len(), 0);
    }

    #[test]
    fn test_add_node() -> Result<()> {
        let mut graph = TSGraph::new();
        let node = NodeData::builder().id("node1").reference_id("chr1").build();

        graph.default_graph_mut().unwrap().add_node(node.clone())?;
        assert_eq!(graph.nodes(DEFAULT_GRAPH_ID).len(), 1);
        assert_eq!(graph.node(DEFAULT_GRAPH_ID, "node1").unwrap().id, node.id);
        Ok(())
    }

    #[test]
    fn test_add_edge() -> Result<()> {
        let mut graph = TSGraph::new();

        // Add nodes first
        let node1 = NodeData::builder().id("node1").reference_id("chr1").build();
        let node2 = NodeData::builder().id("node2").reference_id("chr1").build();

        graph.graph_mut(DEFAULT_GRAPH_ID).unwrap().add_node(node1)?;
        graph.graph_mut(DEFAULT_GRAPH_ID).unwrap().add_node(node2)?;

        // Add edge
        let sv_from_builder = StructuralVariant::builder()
            .reference_name1("chr1")
            .reference_name2("chr1")
            .breakpoint1(1000)
            .breakpoint2(5000)
            .sv_type(BString::from("DEL"))
            .build();
        let edge = EdgeData::builder().id("edge1").sv(sv_from_builder).build();

        graph.graph_mut(DEFAULT_GRAPH_ID).unwrap().add_edge(
            "node1".into(),
            "node2".into(),
            edge.clone(),
        )?;

        assert_eq!(graph.edges(DEFAULT_GRAPH_ID).len(), 1);
        assert_eq!(graph.edge(DEFAULT_GRAPH_ID, "edge1").unwrap().id, edge.id);

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

        // let node = graph.get_node("defaul, "node1").unwrap();
        let node = graph.default_graph().unwrap().node_by_id("node1").unwrap();

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

        let graph_section = graph.default_graph().unwrap();
        assert_eq!(graph_section.groups.len(), 1);
        if let Group::Unordered { id, elements, .. } = &graph_section.groups["group1".as_bytes()] {
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
        assert_eq!(graph.nodes(DEFAULT_GRAPH_ID).len(), 5);
        assert_eq!(graph.edges(DEFAULT_GRAPH_ID).len(), 4);

        graph.to_file("tests/data/test_write.tsg")?;

        Ok(())
    }

    #[test]
    fn test_from_str() -> Result<()> {
        let tsg_string = r#"H	VN	1.0
    H	PN	TestGraph
    N	node1	chr1:+:100-200	read1:SO,read2:IN	ACGT
    N	node2	chr1:+:300-400	read1:SO,read2:IN
    N	node3	chr1:+:500-600	read2:SO,read3:IN
    E	edge1	node1	node2	chr1,chr1,1700,2000,INV
    E	edge2	node2	node3	chr1,chr1,1700,2000,DUP
    C	chain1	node1 edge1 node2 edge2 node3
    "#;

        let graph = TSGraph::from_str(tsg_string)?;

        // Verify headers
        assert_eq!(graph.headers.len(), 2);
        assert_eq!(graph.headers[0].tag, "VN");
        assert_eq!(graph.headers[0].value, "1.0");
        assert_eq!(graph.headers[1].tag, "PN");
        assert_eq!(graph.headers[1].value, "TestGraph");

        // Verify nodes
        assert_eq!(graph.nodes(DEFAULT_GRAPH_ID).len(), 3);
        let node1 = graph.node(DEFAULT_GRAPH_ID, "node1").unwrap();
        assert_eq!(node1.id, "node1");
        assert_eq!(node1.sequence, Some("ACGT".into()));

        // Verify edges
        assert_eq!(graph.edges(DEFAULT_GRAPH_ID).len(), 2);
        let edge1 = graph.edge(DEFAULT_GRAPH_ID, "edge1").unwrap();
        assert_eq!(edge1.id, "edge1");

        // Verify groups
        let graph_section = graph.graph(DEFAULT_GRAPH_ID).unwrap();

        // Verify chain
        if let Group::Chain { elements, .. } = &graph_section.chains["chain1".as_bytes()] {
            assert_eq!(elements.len(), 5);
            assert_eq!(elements[0], "node1");
            assert_eq!(elements[1], "edge1");
        } else {
            panic!("Expected Chain group");
        }

        Ok(())
    }

    #[test]
    fn test_traverse() -> Result<()> {
        let file = "tests/data/test.tsg";
        let graph = TSGraph::from_file(file)?;

        let paths = graph.traverse_by_id(DEFAULT_GRAPH_ID)?;
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

        let dot = graph.to_dot_by_id(DEFAULT_GRAPH_ID, true, true)?;
        println!("{}", dot);

        Ok(())
    }

    #[test]
    fn test_to_json() -> Result<()> {
        let file = "tests/data/test.tsg";
        let graph = TSGraph::from_file(file)?;

        let json = graph.to_json_by_id(DEFAULT_GRAPH_ID)?;
        println!("{}", json);
        Ok(())
    }
}
