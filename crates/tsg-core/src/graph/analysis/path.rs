use crate::graph::TSGPath;
use anyhow::{Context, Result};

#[allow(dead_code)]
pub trait PathAnalysis {
    /// Determines if a path is a "super path" - a path where all nodes share at least one common read
    fn is_super(&self) -> Result<bool>;
}

impl PathAnalysis for TSGPath<'_> {
    /// Determines if a path is a "super path" - a path where all nodes share at least one common read
    ///
    /// A super path indicates that all nodes in the path share at least one common read,
    /// suggesting the path represents a continuous sequence supported by sequencing data.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If all nodes in the path share at least one common read
    /// * `Ok(false)` - If not all nodes share a common read, or if the path has fewer than 2 nodes
    /// * `Err` - If an error occurs during the analysis
    fn is_super(&self) -> Result<bool> {
        // Get the graph reference
        let graph = self.graph().context("Failed to retrieve graph")?;

        // Fast path: If the path has less than 2 nodes, it can't be a super path
        if self.nodes.len() < 2 {
            return Ok(false);
        }

        // Get the first node and its data
        let first_node = &self.nodes[0];

        // If the first node exists, proceed with super path check
        if let Some(first_node_data) = graph.node_weight(*first_node) {
            // Initialize with reads from first node - use capacity hint for better performance
            let mut common_reads = Vec::with_capacity(first_node_data.reads.len());
            for read in &first_node_data.reads {
                common_reads.push(&read.id);
            }

            // Early return if first node has no reads
            if common_reads.is_empty() {
                return Ok(false);
            }

            // Efficiently check each subsequent node for common reads
            for node_idx in &self.nodes[1..] {
                match graph.node_weight(*node_idx) {
                    Some(node_data) => {
                        // Skip the expensive retention check if the node has no reads
                        if node_data.reads.is_empty() {
                            return Ok(false);
                        }

                        // Retain only common reads
                        common_reads
                            .retain(|read_id| node_data.reads.iter().any(|r| &r.id == *read_id));

                        // Early return if no common reads left
                        if common_reads.is_empty() {
                            return Ok(false);
                        }
                    }
                    None => return Ok(false), // Node doesn't exist
                }
            }

            // If we made it here, there is at least one read shared across all nodes
            Ok(true)
        } else {
            Err(anyhow::anyhow!("First node data not found"))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::graph::TSGraph;

    use super::*;

    #[test]
    fn test_is_super_with_common_reads() {
        let tsg_string = r#"H	VN	1.0
        H	PN	TestGraph
        N	node1	chr1:+:100-200	read1:SO,read2:IN	ACGT
        N	node2	chr1:+:300-400	read1:SO,read2:IN
        N	node3	chr1:+:500-600	read2:SO,read3:IN
        E	edge1	node1	node2	chr1,chr1,1700,2000,INV
        E	edge2	node2	node3	chr1,chr1,1700,2000,DUP
        "#;

        let tsgraph = TSGraph::from_str(tsg_string).unwrap();
        let default_graph = tsgraph.default_graph().unwrap();
        let paths = default_graph.traverse().unwrap();

        for path in paths {
            assert!(path.is_super().unwrap());
        }
    }

    #[test]
    fn test_is_super_without_common_reads() {
        let tsg_string = r#"H	VN	1.0
        H	PN	TestGraph
        N	node1	chr1:+:100-200	read1:SO	ACGT
        N	node2	chr1:+:300-400	read2:SO
        N	node3	chr1:+:500-600	read3:SO
        E	edge1	node1	node2	chr1,chr1,1700,2000,INV
        E	edge2	node2	node3	chr1,chr1,1700,2000,DUP
        "#;

        let tsgraph = TSGraph::from_str(tsg_string).unwrap();
        let paths = tsgraph.default_graph().unwrap().traverse().unwrap();

        for path in paths {
            assert!(path.is_super().unwrap());
        }
    }

    #[test]
    fn test_is_super_with_single_node() {
        let tsg_string = r#"H	VN	1.0
        H	PN	TestGraph
        N	node1	chr1:+:100-200	read1:SO,read2:IN	ACGT
        "#;

        let tsgraph = TSGraph::from_str(tsg_string).unwrap();
        let paths = tsgraph.default_graph().unwrap().traverse().unwrap();

        assert_eq!(paths[0].len(), 1);

        for path in paths {
            assert!(!path.is_super().unwrap());
        }
    }

    #[test]
    fn test_is_super_partial_common_reads() {
        let tsg_string = r#"H	VN	1.0
        H	PN	TestGraph
        N	node1	chr1:+:100-200	read1:SO,read2:IN	ACGT
        N	node2	chr1:+:300-400	read1:SO,read3:IN
        N	node3	chr1:+:500-600	read1:SO,read4:IN
        E	edge1	node1	node2	chr1,chr1,1700,2000,INV
        E	edge2	node2	node3	chr1,chr1,1700,2000,DUP
        "#;

        let tsgraph = TSGraph::from_str(tsg_string).unwrap();
        let paths = tsgraph.default_graph().unwrap().traverse().unwrap();

        assert_eq!(&paths[0].len(), &3);

        for path in paths {
            assert!(path.is_super().unwrap());
        }
    }
}
