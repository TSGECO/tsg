use crate::graph::PathAnalysis;
use crate::graph::{GraphSection, TSGraph};
use ahash::HashMap;
use ahash::HashMapExt;
use ahash::HashSet;
use ahash::HashSetExt;
use anyhow::Context;
use anyhow::Result;
use bstr::BString;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::VecDeque;

// TODO this module is not used yet, but it will be used in the future
#[allow(dead_code)]
pub trait GraphAnalysis {
    fn is_connected(&self) -> Result<bool>;
    fn is_cyclic(&self) -> Result<bool>;
    fn is_bubble(&self) -> Result<bool>; // Updated return type to Result<bool>
    fn is_directed_acyclic_graph(&self) -> Result<bool> {
        Ok(self.is_connected()? && !self.is_cyclic()?)
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
        let mut visited = HashSet::new();
        let mut bubbles = Vec::new();

        for start_node in self.node_indices.values() {
            if !visited.contains(start_node) {
                self.find_bubbles(*start_node, &mut bubbles, &mut visited);
            }
        }

        Ok(!bubbles.is_empty())
    }
}

impl GraphSection {
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

    fn find_bubbles(
        &self,
        start: NodeIndex,
        bubbles: &mut Vec<Vec<NodeIndex>>,
        visited: &mut HashSet<NodeIndex>,
    ) {
        // Get all outgoing neighbors
        let outgoing_edges = self._graph.edges(start).collect::<Vec<_>>();

        // If this node has multiple outgoing edges, it might be the start of a bubble
        if outgoing_edges.len() >= 2 {
            // For each pair of outgoing edges
            for i in 0..outgoing_edges.len() {
                let path1_start = outgoing_edges[i].target();

                for outgoing_edge in outgoing_edges.iter().skip(i + 1) {
                    let path2_start = outgoing_edge.target();

                    // Find paths from these two nodes and see if they converge
                    if let Some(bubble) =
                        self.find_convergence_point(start, path1_start, path2_start)
                    {
                        bubbles.push(bubble);
                    }
                }
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

    fn find_convergence_point(
        &self,
        source: NodeIndex,
        path1: NodeIndex,
        path2: NodeIndex,
    ) -> Option<Vec<NodeIndex>> {
        // BFS to find where the two paths converge
        let mut path1_visited = HashMap::new();
        let mut path2_visited = HashMap::new();

        // Initialize queues for BFS
        let mut queue1 = VecDeque::new();
        let mut queue2 = VecDeque::new();

        queue1.push_back(path1);
        path1_visited.insert(path1, vec![source, path1]);

        queue2.push_back(path2);
        path2_visited.insert(path2, vec![source, path2]);

        // Maximum depth to prevent infinite loops
        let max_depth = 100;
        let mut depth = 0;

        while !queue1.is_empty() && !queue2.is_empty() && depth < max_depth {
            depth += 1;

            // Process one level of path1
            if let Some(bubble) =
                self.process_bubble_path(&mut queue1, &mut path1_visited, &path2_visited)
            {
                return Some(bubble);
            };

            // Process one level of path2
            if let Some(bubble) =
                self.process_bubble_path(&mut queue2, &mut path2_visited, &path1_visited)
            {
                return Some(bubble);
            }
        }

        None
    }

    fn process_bubble_path(
        &self,
        queue: &mut VecDeque<NodeIndex>,
        visited: &mut HashMap<NodeIndex, Vec<NodeIndex>>,
        other_visited: &HashMap<NodeIndex, Vec<NodeIndex>>,
    ) -> Option<Vec<NodeIndex>> {
        if queue.is_empty() {
            return None;
        }

        let node = queue.pop_front().unwrap();
        let current_path = visited.get(&node).unwrap().clone();

        // Check if this node has been visited in the other path - convergence point
        if other_visited.contains_key(&node) {
            // Found a bubble - combine the paths
            let mut bubble = current_path.clone();
            let mut other_path = other_visited.get(&node).unwrap().clone();

            // Ensure the bubble doesn't duplicate the convergence point
            other_path.pop();
            other_path.reverse();
            bubble.extend(other_path);

            return Some(bubble);
        }

        // Continue BFS
        for edge in self._graph.edges(node) {
            let next = edge.target();
            if let std::collections::hash_map::Entry::Vacant(e) = visited.entry(next) {
                let mut new_path = current_path.clone();
                new_path.push(next);
                e.insert(new_path);
                queue.push_back(next);
            }
        }

        None
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
E	edge6	node1	node3	chr1,chr1,1700,2000,INV
"#;

        let tsgraph = TSGraph::from_str(tsg_string).unwrap();
        let graph = tsgraph.default_graph().unwrap();
        let bubbles = graph.is_bubble().unwrap();
        assert!(bubbles);

        // No bubbles in a linear graph
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
        let bubbles = graph.is_bubble().unwrap();
        assert!(!bubbles);
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
}
