use crate::graph::{GraphSection, PathAnalysis, TSGraph};
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use anyhow::{Context, Ok, Result};
use bstr::BString;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::VecDeque;

/// Enumeration representing different graph topologies.
/// The topology can be used to classify the structure of the graph.
/// One graph only include one topology.
#[derive(Debug, Clone)]
pub enum GraphTopology {
    /// The graph is a fade-in structure.
    FadeIn,
    /// The graph is a fade-out structure.
    FadeOut,
    /// The graph is bipartite.
    Bipartite,

    /// The graph is a unique path.
    UniquePath,
    /// The graph is an equi-path.
    EquiPath,
    /// The graph is a hetero-path.
    HeteroPath,

    /// The category is not defined.
    NotDefined,
}

pub trait GraphAnalysis {
    fn topo(&self) -> Result<GraphTopology>;

    /// Determines whether the graph is connected.
    ///
    /// A graph is connected if there is a path between every pair of vertices.
    /// For directed graphs, this considers connections in both directions.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the graph is connected or empty
    /// * `Ok(false)` - If the graph has disconnected components
    /// * `Err` - If an error occurs during the analysis
    fn is_connected(&self) -> Result<bool>;

    /// Determines whether the graph contains any cycles.
    ///
    /// A cycle is a path that starts and ends at the same node.
    /// This method performs a depth-first search to detect cycles.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the graph contains at least one cycle
    /// * `Ok(false)` - If the graph is acyclic (contains no cycles)
    /// * `Err` - If an error occurs during the analysis
    fn is_cyclic(&self) -> Result<bool>;

    /// Determines whether the graph contains any bubbles.
    ///
    /// A bubble is a subgraph that starts at a single source node, branches into multiple paths,
    /// and then reconverges at a single sink node.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the graph contains at least one bubble
    /// * `Ok(false)` - If the graph does not contain any bubbles
    /// * `Err` - If an error occurs during the analysis
    fn is_bubble(&self) -> Result<bool>;

    /// Determines whether the graph is a directed acyclic graph (DAG).
    ///
    /// A graph is a DAG if it is both connected and does not contain cycles.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the graph is a DAG
    /// * `Ok(false)` - If the graph is not a DAG
    /// * `Err` - If an error occurs during the analysis
    fn is_directed_acyclic_graph(&self) -> Result<bool> {
        Ok(self.is_connected()? && !self.is_cyclic()?)
    }

    /// Determines whether the graph is simple.
    ///
    /// A graph is considered simple if the maximum path length is 1.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the graph is simple
    /// * `Ok(false)` - If the graph is not simple
    /// * `Err` - If an error occurs during the analysis
    fn is_simple(&self) -> Result<bool>;

    /// Determines whether the directed graph is a fade-in structure.
    ///
    /// A graph is considered a fade-in if it is simple and has only one source node.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the graph is a fade-in structure
    /// * `Ok(false)` - If the graph is not a fade-in structure
    /// * `Err` - If an error occurs during the analysis
    fn is_fade_in(&self) -> Result<bool> {
        Ok(matches!(self.topo()?, GraphTopology::FadeIn))
    }

    /// Determines whether the directed graph is a fade-out structure.
    ///
    /// A graph is considered a fade-out if it is simple and has only one sink node.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the graph is a fade-out structure
    /// * `Ok(false)` - If the graph is not a fade-out structure
    /// * `Err` - If an error occurs during the analysis
    fn is_fade_out(&self) -> Result<bool> {
        Ok(matches!(self.topo()?, GraphTopology::FadeOut))
    }

    /// Determines whether the graph is bipartite.
    ///
    /// A bipartite graph is a graph which is simple but not fade-in and fade-out
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the graph is bipartite
    /// * `Ok(false)` - If the graph is not bipartite
    /// * `Err` - If an error occurs during the analysis
    fn is_bipartite(&self) -> Result<bool> {
        Ok(matches!(self.topo()?, GraphTopology::Bipartite))
    }

    /// Determines whether the graph contains a unique path.
    ///
    /// A graph has a unique path if it is not simple and there is only one path
    /// after traversing the graph.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the graph contains a unique path
    /// * `Ok(false)` - If the graph does not contain a unique path
    /// * `Err` - If an error occurs during the analysis
    fn is_unique_path(&self) -> Result<bool> {
        Ok(matches!(self.topo()?, GraphTopology::UniquePath))
    }

    /// Determines whether the graph contains equi-paths.
    ///
    /// A graph has equi-paths if it is not simple, contains bubbles (alternative paths),
    /// and all alternative paths have the same length.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the graph contains equi-paths
    /// * `Ok(false)` - If the graph does not contain equi-paths
    /// * `Err` - If an error occurs during the analysis
    fn is_equi_path(&self) -> Result<bool> {
        Ok(matches!(self.topo()?, GraphTopology::EquiPath))
    }

    /// Determines whether the graph contains hetero-paths.
    ///
    /// A graph has hetero-paths if it is not simple, contains bubbles (alternative paths),
    /// and the alternative paths have different lengths.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the graph contains hetero-paths
    /// * `Ok(false)` - If the graph does not contain hetero-paths
    /// * `Err` - If an error occurs during the analysis
    fn is_hetero_path(&self) -> Result<bool> {
        Ok(matches!(self.topo()?, GraphTopology::HeteroPath))
    }

    /// Generates a summary of the graph's properties.
    ///
    /// # Returns
    ///
    /// * `Ok(BString)` - A summary of the graph's properties
    /// * `Err` - If an error occurs during the summarization
    fn summarize(&self) -> Result<BString>;
}

impl GraphAnalysis for GraphSection {
    fn is_connected(&self) -> Result<bool> {
        if self.nodes().is_empty() {
            return Ok(true); // Empty graph is trivially connected
        }

        // Start DFS from the first node
        let start_node = self.node_indices.values().next().unwrap();
        let mut visited = HashSet::new();

        // Perform DFS to find all reachable nodes
        self.dfs(*start_node, &mut visited);

        // The graph is connected if all nodes are visited
        Ok(visited.len() == self.node_indices.len())
    }

    fn is_cyclic(&self) -> Result<bool> {
        // Track both visited nodes and nodes in the current recursion stack
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        // Check from each node (to handle disconnected components)
        for start_node in self.node_indices.values() {
            if self.is_cyclic_util(*start_node, &mut visited, &mut rec_stack) {
                return Ok(true);
            }
        }

        Ok(false) // Updated to return Result<bool>
    }

    fn is_bubble(&self) -> Result<bool> {
        let bubbles = self.collect_bubbles()?;
        Ok(!bubbles.is_empty())
    }

    fn is_simple(&self) -> Result<bool> {
        // A graph is simple if the maximum path length is 1
        let paths = self.traverse()?;
        let max_path_len = paths.iter().map(|path| path.len()).max().unwrap_or(0);
        Ok(max_path_len == 1)
    }

    fn topo(&self) -> Result<GraphTopology> {
        // Check if the graph is simple first since we need this for classification
        let is_simple = self.is_simple()?;

        // Count sources and sinks only once
        let sources = self
            ._graph
            .node_indices()
            .filter(|&n| {
                self._graph
                    .edges_directed(n, petgraph::Direction::Incoming)
                    .count()
                    == 0
            })
            .count();

        let sinks = self
            ._graph
            .node_indices()
            .filter(|&n| self._graph.edges(n).count() == 0)
            .count();

        if is_simple {
            // Simple graph classification
            match (sources, sinks) {
                (s, 1) if s > 1 => Ok(GraphTopology::FadeIn),
                (1, s) if s > 1 => Ok(GraphTopology::FadeOut),
                (s, t) if s > 1 && t > 1 => Ok(GraphTopology::Bipartite),
                _ => Ok(GraphTopology::NotDefined),
            }
        } else {
            // Handle non-simple graph
            if sources == 1 && sinks == 1 {
                return Ok(GraphTopology::UniquePath);
            }

            // Only collect bubbles if needed
            let bubbles = self.collect_bubbles()?;

            if bubbles.is_empty() {
                return Ok(GraphTopology::NotDefined);
            }

            // Check if any bubble has paths of different lengths
            for bubble in &bubbles {
                if bubble[0].len() != bubble[1].len() {
                    return Ok(GraphTopology::HeteroPath);
                }
            }

            Ok(GraphTopology::EquiPath)
        }
    }

    fn summarize(&self) -> Result<BString> {
        unimplemented!()
    }
}

impl GraphSection {
    /// Performs a depth-first search (DFS) traversal of the graph.
    ///
    /// This method visits nodes in the graph in a depth-first manner, marking each visited node.
    /// It considers both outgoing and incoming edges to ensure connectivity in both directions,
    /// which is necessary for undirected connectivity analysis.
    ///
    /// # Parameters
    ///
    /// * `node` - The current node being visited in the traversal
    /// * `visited` - A mutable HashSet tracking which nodes have been visited to avoid cycles
    ///
    /// # Note
    ///
    /// The method modifies the `visited` set in-place, adding each node encountered during traversal.
    /// This is primarily used by the `is_connected` method to determine graph connectivity.
    fn dfs(&self, node: NodeIndex, visited: &mut HashSet<NodeIndex>) {
        // If already visited, return
        if visited.contains(&node) {
            return;
        }

        // Mark as visited
        visited.insert(node);

        // Visit all neighbors through outgoing edges
        for edge in self._graph.edges(node) {
            self.dfs(edge.target(), visited);
        }

        // Visit all neighbors through incoming edges
        // This is necessary for undirected connectivity
        for edge in self
            ._graph
            .edges_directed(node, petgraph::Direction::Incoming)
        {
            self.dfs(edge.source(), visited);
        }
    }

    /// Helper method for cycle detection in a graph.
    ///
    /// This method implements a depth-first search that tracks both visited nodes
    /// and nodes currently in the recursion stack to detect cycles.
    ///
    /// # Parameters
    ///
    /// * `node` - The current node being visited in the traversal
    /// * `visited` - A mutable HashSet tracking all nodes that have been visited
    /// * `rec_stack` - A mutable HashSet tracking nodes in the current recursion path
    ///
    /// # Returns
    ///
    /// `true` if a cycle is detected, `false` otherwise
    fn is_cyclic_util(
        &self,
        node: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
        rec_stack: &mut HashSet<NodeIndex>,
    ) -> bool {
        // If node is not visited yet, mark it visited and add to recursion stack
        if !visited.contains(&node) {
            visited.insert(node);
            rec_stack.insert(node);

            // Visit all neighbors through outgoing edges
            for edge in self._graph.edges(node) {
                let neighbor = edge.target();

                // If neighbor is not visited, check if cycles exist in DFS
                if !visited.contains(&neighbor) {
                    if self.is_cyclic_util(neighbor, visited, rec_stack) {
                        return true;
                    }
                }
                // If neighbor is in recursion stack, we found a cycle
                else if rec_stack.contains(&neighbor) {
                    return true;
                }
            }
        }

        // Remove node from recursion stack when we're done exploring it
        rec_stack.remove(&node);
        false
    }

    fn collect_bubbles(&self) -> Result<Vec<Vec<Vec<NodeIndex>>>> {
        let mut visited = HashSet::new();
        let mut bubble_pairs = Vec::new();

        for start_node in self.node_indices.values() {
            if !visited.contains(start_node) {
                self.find_bubbles(*start_node, &mut bubble_pairs, &mut visited);
            }
        }
        Ok(bubble_pairs)
    }

    fn find_bubbles(
        &self,
        start: NodeIndex,
        bubbles: &mut Vec<Vec<Vec<NodeIndex>>>,
        visited: &mut HashSet<NodeIndex>,
    ) {
        // Get all outgoing neighbors
        let outgoing_edges = self._graph.edges(start).collect::<Vec<_>>();

        // If this node has multiple outgoing edges, it might be the start of a bubble
        if outgoing_edges.len() >= 2 {
            // For each pair of outgoing edges, check if they lead to the same end node
            for i in 0..outgoing_edges.len() {
                let path1_start = outgoing_edges[i].target();

                for j in i + 1..outgoing_edges.len() {
                    let path2_start = outgoing_edges[j].target();

                    // Find bubbles from these two starting points
                    self.find_bubble_paths(start, path1_start, path2_start, bubbles);
                }
            }

            // Check for direct edges and alternative paths that form bubbles
            let direct_targets: HashSet<NodeIndex> =
                outgoing_edges.iter().map(|e| e.target()).collect();

            for target in &direct_targets {
                // For each direct target, check if there are alternative paths to it
                self.check_alternative_paths(start, *target, &direct_targets, bubbles);
            }
        }

        // Mark current node as visited
        visited.insert(start);

        // Continue DFS for bubble detection
        for edge in outgoing_edges {
            let next_node = edge.target();
            if !visited.contains(&next_node) {
                self.find_bubbles(next_node, bubbles, visited);
            }
        }
    }

    // Helper method to find bubble paths between two starting nodes
    fn find_bubble_paths(
        &self,
        source: NodeIndex,      // The common source node
        path1_start: NodeIndex, // First path's start node
        path2_start: NodeIndex, // Second path's start node
        bubbles: &mut Vec<Vec<Vec<NodeIndex>>>,
    ) {
        // Track visited nodes and their paths for each branch
        let mut path1_visited = HashMap::new();
        let mut path2_visited = HashMap::new();

        // Track nodes where both paths converge (potential bubble end points)
        let mut convergence_points = HashSet::new();

        // Initialize queues for BFS
        let mut queue1 = VecDeque::new();
        let mut queue2 = VecDeque::new();

        queue1.push_back(path1_start);
        path1_visited.insert(path1_start, vec![source, path1_start]);

        queue2.push_back(path2_start);
        path2_visited.insert(path2_start, vec![source, path2_start]);

        // BFS to find all possible convergence points
        let max_depth = 100; // Prevent infinite loops
        let mut depth = 0;

        while (!queue1.is_empty() || !queue2.is_empty()) && depth < max_depth {
            depth += 1;

            // Process one level of path1
            self.process_path(
                &mut queue1,
                &mut path1_visited,
                &mut path2_visited,
                &mut convergence_points,
            );

            // Process one level of path2
            self.process_path(
                &mut queue2,
                &mut path2_visited,
                &mut path1_visited,
                &mut convergence_points,
            );

            // If we found convergence points, create bubble pairs
            if !convergence_points.is_empty() {
                // For each convergence point, construct a bubble pair
                for &end_point in &convergence_points {
                    if let Some(path1) = path1_visited.get(&end_point) {
                        if let Some(path2) = path2_visited.get(&end_point) {
                            // We have two paths that start at source and end at end_point
                            // This is a proper bubble with common start and end points

                            // Create a bubble pair if both paths are valid and different
                            if path1.len() >= 3
                                && path2.len() >= 3
                                && path1.first() == Some(&source)
                                && path1.last() == Some(&end_point)
                                && path2.first() == Some(&source)
                                && path2.last() == Some(&end_point)
                                && path1 != path2
                            {
                                // Create a bubble pair as a Vec of two paths
                                let bubble_pair = vec![path1.clone(), path2.clone()];
                                bubbles.push(bubble_pair);
                            }
                        }
                    }
                }

                // We found bubbles at this level, so we're done
                break;
            }
        }
    }

    // Helper to process one level of a path during bubble search
    fn process_path(
        &self,
        queue: &mut VecDeque<NodeIndex>,
        current_visited: &mut HashMap<NodeIndex, Vec<NodeIndex>>,
        other_visited: &HashMap<NodeIndex, Vec<NodeIndex>>,
        convergence_points: &mut HashSet<NodeIndex>,
    ) {
        if queue.is_empty() {
            return;
        }

        let node = queue.pop_front().unwrap();
        let current_path = current_visited.get(&node).unwrap().clone();

        // Check if this node has been visited in the other path - convergence point
        if other_visited.contains_key(&node) {
            // Found a convergence point - this is a potential bubble end point
            convergence_points.insert(node);
            return;
        }

        // Continue BFS
        for edge in self._graph.edges(node) {
            let next = edge.target();
            if let std::collections::hash_map::Entry::Vacant(e) = current_visited.entry(next) {
                let mut new_path = current_path.clone();
                new_path.push(next);
                e.insert(new_path);
                queue.push_back(next);
            }
        }
    }

    /// Check for alternative paths between a start node and a target node
    /// A true bubble must have both a common start point and a common end point
    fn check_alternative_paths(
        &self,
        start: NodeIndex,
        target: NodeIndex,
        direct_targets: &HashSet<NodeIndex>,
        bubbles: &mut Vec<Vec<Vec<NodeIndex>>>,
    ) {
        // First, check if there is a direct path from start to target
        let direct_path = vec![start, target];

        // Next, find all alternative paths from start to target
        let mut alternative_paths = Vec::new();

        // BFS to find all paths from start to target
        let mut queue = VecDeque::new();
        let mut paths: HashMap<NodeIndex, Vec<Vec<NodeIndex>>> = HashMap::new();

        // Initialize with all direct neighbors except the target
        for edge in self._graph.edges(start) {
            let next = edge.target();
            if next != target {
                queue.push_back(next);
                paths.insert(next, vec![vec![start, next]]);
            }
        }

        // Track visited nodes to avoid cycles
        let mut visited = HashSet::new();
        visited.insert(start);

        // BFS with path tracking
        while let Some(node) = queue.pop_front() {
            if visited.contains(&node) {
                continue;
            }

            visited.insert(node);
            let current_paths = paths.get(&node).unwrap().clone();

            for edge in self._graph.edges(node) {
                let next = edge.target();

                // If we reached our target, we found an alternative path
                if next == target {
                    for path in &current_paths {
                        let mut bubble_path = path.clone();
                        bubble_path.push(target);

                        // Only add as an alternative path if it's valid:
                        // 1. Path must start at the start node
                        // 2. Path must end at the target node
                        // 3. Path must be at least 3 nodes long (start->middle->target)
                        if bubble_path.len() >= 3
                            && bubble_path.first() == Some(&start)
                            && bubble_path.last() == Some(&target)
                        {
                            alternative_paths.push(bubble_path);
                        }
                    }
                    continue;
                }

                // Skip if we've seen this node already to avoid cycles
                if visited.contains(&next) || direct_targets.contains(&next) {
                    continue;
                }

                // Create new paths by extending current paths
                let mut new_paths = Vec::new();
                for path in &current_paths {
                    let mut new_path = path.clone();
                    new_path.push(next);
                    new_paths.push(new_path);
                }

                // Update or insert paths for this node
                paths
                    .entry(next)
                    .and_modify(|e| e.extend(new_paths.clone()))
                    .or_insert(new_paths.clone());

                // Add to queue for further exploration
                queue.push_back(next);
            }
        }

        // If there's a direct path from start to target AND at least one alternative path,
        // create bubble pairs
        if !alternative_paths.is_empty() {
            // For each alternative path, create a bubble pair with the direct path
            for alt_path in alternative_paths {
                // Create a bubble pair as a Vec of two paths
                let bubble_pair = vec![direct_path.clone(), alt_path];
                bubbles.push(bubble_pair);
            }
        }
    }
}

pub trait TSGraphAnalysis {
    fn summarize(&self) -> Result<BString>;
}

impl TSGraphAnalysis for TSGraph {
    fn summarize(&self) -> Result<BString> {
        // Pre-calculate capacity based on expected size
        let graph_count = self.graphs.len();
        // Estimate 30 bytes per graph entry (adjust as needed based on actual data sizes)
        let estimated_capacity = graph_count * 30;

        // Pre-allocate with capacity to avoid reallocations
        let mut summary = Vec::with_capacity(estimated_capacity);
        let headers = [
            "gid",
            "nodes",
            "edges",
            "paths",
            "max_path_len",
            "super_path",
            "bubble",
        ];

        let delimiter = ",";
        let header_str = headers.join(delimiter) + "\n";
        summary.extend_from_slice(header_str.as_bytes());

        for (id, graph) in self.graphs.iter() {
            let node_count = graph.nodes().len();
            let edge_count = graph.edges().len();
            let paths = graph.traverse()?;

            let path_count = paths.len();
            let max_path_len = paths.iter().map(|path| path.nodes.len()).max().unwrap_or(0);

            let include_super_path = paths.iter().any(|path| {
                path.is_super()
                    .context("Failed to check super path")
                    .unwrap()
            });
            let graph_is_bubble = graph.is_bubble()?;

            // Use write! to format directly into the buffer without intermediate allocations
            use std::io::Write;
            writeln!(
                summary,
                "{},{},{},{},{},{},{}",
                id,
                node_count,
                edge_count,
                path_count,
                max_path_len,
                include_super_path,
                graph_is_bubble
            )?;
        }
        // Convert to BString only once at the end
        Ok(BString::from(summary))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::TSGraph;
    use std::str::FromStr;

    #[test]
    fn test_is_connected() {
        // Create a connected graph
        let tsg_string = r#"H	VN	1.0
H	PN	TestGraph
N	node1	chr1:+:100-200	read1:SO,read2:IN	ACGT
N	node2	chr1:+:300-400	read1:SO,read3:IN
N	node3	chr1:+:500-600	read1:SO,read4:IN
E	edge1	node1	node2	chr1,chr1,1700,2000,INV
E	edge2	node2	node3	chr1,chr1,1700,2000,DUP
"#;

        let tsgraph = TSGraph::from_str(tsg_string).unwrap();
        let graph = tsgraph.default_graph().unwrap();
        assert!(graph.is_connected().unwrap());

        // Create a disconnected graph
        let tsg_string = r#"H	VN	1.0
H	PN	TestGraph
N	node1	chr1:+:100-200	read1:SO,read2:IN	ACGT
N	node2	chr1:+:300-400	read1:SO,read3:IN
N	node3	chr1:+:500-600	read1:SO,read4:IN
E	edge1	node1	node2	chr1,chr1,1700,2000,INV
"#;

        let tsgraph = TSGraph::from_str(tsg_string).unwrap();
        let graph = tsgraph.default_graph().unwrap();
        assert!(!graph.is_connected().unwrap());
    }

    #[test]
    fn test_is_cyclic() {
        // Create an acyclic graph
        let tsg_string = r#"H	VN	1.0
H	PN	TestGraph
N	node1	chr1:+:100-200	read1:SO,read2:IN	ACGT
N	node2	chr1:+:300-400	read1:SO,read3:IN
N	node3	chr1:+:500-600	read1:SO,read4:IN
E	edge1	node1	node2	chr1,chr1,1700,2000,INV
E	edge2	node2	node3	chr1,chr1,1700,2000,DUP
"#;
        let tsgraph = TSGraph::from_str(tsg_string).unwrap();
        let graph = tsgraph.default_graph().unwrap();
        assert!(!graph.is_cyclic().unwrap());

        // Create a cyclic graph
        let tsg_string = r#"H	VN	1.0
H	PN	TestGraph
N	node1	chr1:+:100-200	read1:SO,read2:IN	ACGT
N	node2	chr1:+:300-400	read1:SO,read3:IN
N	node3	chr1:+:500-600	read1:SO,read4:IN
E	edge1	node1	node2	chr1,chr1,1700,2000,INV
E	edge2	node2	node3	chr1,chr1,1700,2000,DUP
E	edge3	node3	node1	chr1,chr1,1700,2000,DUP
"#;
        let tsgraph = TSGraph::from_str(tsg_string).unwrap();
        let graph = tsgraph.default_graph().unwrap();

        assert!(graph.is_cyclic().unwrap());
    }

    #[test]
    fn test_detect_bubbles() {
        // Create a graph with a bubble
        let tsg_string = r#"H	VN	1.0
H	PN	TestGraph
N	node1	chr1:+:100-200	read1:SO,read2:IN	ACGT
N	node2	chr1:+:300-400	read1:SO,read3:IN
N	node3	chr1:+:500-600	read1:SO,read4:IN
N	node4	chr1:+:700-800	read1:SO,read5:IN
E	edge1	node1	node2	chr1,chr1,1700,2000,INV
E	edge2	node2	node3	chr1,chr1,1700,2000,DUP
E	edge3	node2	node4	chr1,chr1,1700,2000,DUP
E	edge4	node3	node4	chr1,chr1,1700,2000,INV
E	edge5	node1	node3	chr1,chr1,1700,2000,INV
"#;

        let tsgraph = TSGraph::from_str(tsg_string).unwrap();
        let graph = tsgraph.default_graph().unwrap();
        let is_bubble = graph.is_bubble().unwrap();
        assert!(is_bubble);

        let bubbles = graph.collect_bubbles().unwrap();
        println!("Bubbles: {:?}", bubbles);

        // Verify the bubbles are detected as pairs of paths
        assert!(!bubbles.is_empty(), "Should detect at least one bubble");

        // Each bubble should be a pair of alternative paths
        for bubble_pair in &bubbles {
            assert_eq!(
                bubble_pair.len(),
                2,
                "Each bubble should have exactly 2 paths"
            );

            // Both paths in a pair should have the same start and end nodes
            let path1 = &bubble_pair[0];
            let path2 = &bubble_pair[1];

            assert_eq!(
                path1.first(),
                path2.first(),
                "Paths should have the same start node"
            );
            assert_eq!(
                path1.last(),
                path2.last(),
                "Paths should have the same end node"
            );
            assert!(path1 != path2, "The two paths should be different");
        }

        // No bubbles in a linear graph
        let tsg_string = r#"H	VN	1.0
H	PN	TestGraph
N	node1	chr1:+:100-200	read1:SO,read2:IN	ACGT
N	node2	chr1:+:300-400	read1:SO,read3:IN
N	node3	chr1:+:500-600	read1:SO,read4:IN
E	edge1	node1	node2	chr1,chr1,1700,2000,INV
E	edge2	node2	node3	chr1,chr1,1700,2000,DUP
"#;

        let tsg_graph = TSGraph::from_str(tsg_string).unwrap();
        let graph = tsg_graph.default_graph().unwrap();
        let is_bubble = graph.is_bubble().unwrap();
        assert!(!is_bubble, "Should not detect bubbles in a linear graph");
    }

    #[test]
    fn test_summarize() {
        let tsg_string = r#"H	VN	1.0
    H	PN	TestGraph
    G	g1
    N	node1	chr1:+:100-200	read1:SO,read2:IN	ACGT
    N	node2	chr1:+:300-400	read1:SO,read3:IN
    N	node3	chr1:+:500-600	read1:SO,read4:IN
    E	edge1	node1	node2	chr1,chr1,1700,2000,INV
    E	edge2	node2	node3	chr1,chr1,1700,2000,DUP
    G	g2
    N	node4	chr2:+:100-200	read5:SO,read6:IN	ACGT
    N	node5	chr2:+:300-400	read5:SO,read7:IN
    E	edge3	node4	node5	chr2,chr2,1700,2000,INV
    "#;

        let tsgraph = TSGraph::from_str(tsg_string).unwrap();
        let summary = tsgraph.summarize().unwrap();
        let summary_str = summary.to_string();

        assert!(summary_str.contains("edges"));
        assert!(summary_str.contains("paths"));
        assert!(summary_str.contains("max_path_len"));
        assert!(summary_str.contains("g1"));
        assert!(summary_str.contains("g2"));
        assert!(summary_str.contains("edges"));
        assert!(summary_str.contains("paths"));
    }

    #[test]
    fn test_proper_bubble_detection() {
        // Create a graph with the example from the prompt
        let tsg_string = r#"H	VN	1.0
H	PN	TestGraph
N	node1	chr1:+:100-200	read1:SO,read2:IN	ACGT
N	node2	chr1:+:300-400	read1:SO,read3:IN
N	node3	chr1:+:500-600	read1:SO,read4:IN
N	node4	chr1:+:700-800	read1:SO,read5:IN
E	edge1	node1	node2	chr1,chr1,1700,2000,INV
E	edge2	node2	node3	chr1,chr1,1700,2000,DUP
E	edge3	node2	node4	chr1,chr1,1700,2000,DUP
E	edge4	node3	node4	chr1,chr1,1700,2000,INV
E	edge5	node1	node3	chr1,chr1,1700,2000,INV
"#;

        let tsgraph = TSGraph::from_str(tsg_string).unwrap();
        let graph = tsgraph.default_graph().unwrap();

        // Collect all bubbles in the graph
        let bubbles = graph.collect_bubbles().unwrap();
        // Extract node names for test validation
        let node_names: HashMap<_, _> = graph.node_indices_to_ids();

        // Print the bubbles with node names for debugging
        println!("Found {} bubbles:", bubbles.len());
        for (i, bubble) in bubbles.iter().enumerate() {
            let path1 = bubble[0]
                .iter()
                .map(|&idx| node_names.get(&idx).unwrap().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");

            let path2 = bubble[1]
                .iter()
                .map(|&idx| node_names.get(&idx).unwrap().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            println!("Bubble {}: {} and {}", i + 1, path1, path2);
        }

        // Check that we only detect proper bubbles (paths with both common start and end points)
        // We should NOT detect "node1 -> node2 -> node3" or "node2 -> node3 -> node4" as bubbles
        // because they don't have common end points

        // True bubbles should include:
        // 1. node1 -> node2 -> node4 and node1 -> node3 -> node4 (common start node1, common end node4)
        // 2. node1 -> node2 -> node3 and node1 -> node3 (common start node1, common end node3)
        // 3. node2 -> node3 -> node4 and node2 -> node4 (common start node2, common end node4)

        assert_eq!(bubbles.len(), 3, "Should detect exactly 3 bubbles");
    }
}
