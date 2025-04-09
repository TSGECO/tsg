use crate::graph::TSGPath;
use anyhow::Context;
use anyhow::Result;

#[allow(dead_code)]
pub trait PathAnalysis {
    fn is_super(&self) -> Result<bool>;
}

impl PathAnalysis for TSGPath<'_> {
    fn is_super(&self) -> Result<bool> {
        // check if the graph is empty. raise an error if it is
        let graph = self.graph().context("Failed to retrieve graph")?;

        // check if nodes in the path share at least one read (reads from node data)
        // If the path has less than 2 nodes, it can't be a super path
        if self.nodes.len() < 2 {
            return Ok(false);
        }

        // Get the reads from the first node
        let first_node = &self.nodes[0];

        if let Some(first_node_data) = graph.node_weight(*first_node) {
            let mut common_reads = first_node_data
                .reads
                .iter()
                .map(|x| &x.id)
                .collect::<Vec<_>>();

            // Check each subsequent node for common reads
            for node_idx in &self.nodes[1..] {
                if let Some(node_data) = graph.node_weight(*node_idx) {
                    // Keep only the reads that are common to both the current node and our running set
                    common_reads
                        .retain(|read_id| node_data.reads.iter().any(|r| &r.id == *read_id));

                    // If there are no common reads left, return Ok(false)
                    if common_reads.is_empty() {
                        return Ok(false);
                    }
                } else {
                    // If a node doesn't exist, return Ok(false)
                    return Ok(false);
                }
            }

            // If we made it here, there is at least one read shared across all nodes
            return Ok(!common_reads.is_empty());
        }

        Err(anyhow::anyhow!("First node data not found"))
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
        let paths = tsgraph.default_graph().unwrap().traverse().unwrap();

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
