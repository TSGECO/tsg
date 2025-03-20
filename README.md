<img src="./docs/logo.png" alt="crates.io" height="100" align="center"/>

# TSG - Transcript Segment Graph

TSG is a Rust library and command-line tool for creating, manipulating, and analyzing transcript segment graphs. It provides a comprehensive framework for modeling segmented transcript data, analyzing non-linear splicing events, and working with genomic structural variants.

## Features

- Parse and write TSG format files
- Build and manipulate transcript segment graphs
- Support for multiple graphs within a single file
- Analyze paths and connectivity between transcript segments
- Support for various element types: nodes, edges, groups, and chains
- Export graphs to DOT format for visualization
- Traverse the graph to identify valid transcript paths
- Read identity tracking to ensure biological validity
- Build graphs from chains and validate path traversals
- Support for genomic coordinates with strand information
- Support for read evidence with types
- Inter-graph links for fusion events and other cross-graph relationships

## Installation

### Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
tsg = "0.1.0"
```

### Command-line Tool

Install the CLI tool:

```bash
cargo install tsg
```

## Library Usage

### Loading a TSG file

```rust
use tsg::graph::TSGraph;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load graph from a TSG file
    let graph = TSGraph::from_file("path/to/file.tsg")?;

    // Access graph elements
    println!("Number of graphs: {}", graph.get_graphs().len());
    println!("Number of nodes: {}", graph.get_nodes().len());
    println!("Number of edges: {}", graph.get_edges().len());

    // Export to DOT format for visualization
    let dot = graph.to_dot()?;
    std::fs::write("graph.dot", dot)?;

    // Save modified graph
    graph.write_to_file("output.tsg")?;

    Ok(())
}
```

### Working with Multiple Graphs

```rust
use tsg::graph::{TSGraph, NodeData, EdgeData};
use bstr::BString;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = TSGraph::new();

    // Define multiple graphs
    graph.add_graph("gene_a", Some("BRCA1 transcripts"))?;
    graph.add_graph("gene_b", Some("BRCA2 transcripts"))?;

    // Add nodes to different graphs
    let node1 = NodeData {
        id: "gene_a:n1".into(),
        reference_id: "chr17".into(),
        ..Default::default()
    };

    let node2 = NodeData {
        id: "gene_b:n1".into(),
        reference_id: "chr13".into(),
        ..Default::default()
    };

    graph.add_node(node1)?;
    graph.add_node(node2)?;

    // Add edges within each graph
    let edge1 = EdgeData {
        id: "gene_a:e1".into(),
        ..Default::default()
    };

    graph.add_edge("gene_a:n1".into(), "gene_a:n2".into(), edge1)?;

    // Add inter-graph link (e.g., for fusion transcript)
    graph.add_link("fusion1", "gene_a:n3", "gene_b:n1", "fusion", None)?;

    // Write to file
    graph.write_to_file("multi_graph.tsg")?;

    Ok(())
}
```

### Building Graphs from Chains

```rust
use tsg::graph::{TSGraph, Group};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create chains for different graphs
    let chains = vec![
        Group::Chain {
            id: "gene_a:chain1".into(),
            elements: vec!["gene_a:n1".into(), "gene_a:e1".into(), "gene_a:n2".into()],
            attributes: HashMap::new(),
        },
        Group::Chain {
            id: "gene_b:chain1".into(),
            elements: vec!["gene_b:n1".into(), "gene_b:e1".into(), "gene_b:n2".into()],
            attributes: HashMap::new(),
        },
    ];

    // Build graphs from chains
    let graph = TSGraph::from_chains(chains)?;

    // Write to file
    graph.write_to_file("output.tsg")?;

    Ok(())
}
```

### Finding Valid Paths Through Specific Graphs

```rust
use tsg::graph::TSGraph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let graph = TSGraph::from_file("transcript.tsg")?;

    // Find all valid paths through a specific graph
    let paths = graph.traverse_graph("gene_a")?;

    for (i, path) in paths.iter().enumerate() {
        println!("Path {}: {}", i+1, path);
    }

    Ok(())
}
```

## CLI Usage

The TSG command-line tool provides a convenient interface for common operations:

```bash
# Display help
tsg --help

# Parse and validate a TSG file
tsg validate path/to/file.tsg

# List all graphs in a TSG file
tsg list-graphs path/to/file.tsg

# Convert a specific graph to DOT format for visualization
tsg dot --graph=gene_a path/to/file.tsg > gene_a.dot

# Extract statistics from a TSG file
tsg stats path/to/file.tsg

# Find all paths through a specific graph
tsg paths --graph=gene_a path/to/file.tsg

# Find all inter-graph links
tsg links path/to/file.tsg
```

## TSG File Format

The TSG format is a tab-delimited text format representing transcript assemblies as graphs. It supports multiple independent graphs within a single file.

### Multi-Graph Support

TSG supports multiple graphs within a single file using a graph namespace approach. Each element in the file can be associated with a specific graph using a graph ID prefix:

```
graph_id:element_id
```

For example, `gene_a:n1` refers to node n1 in the graph identified as "gene_a".

### Record Types

Each line in a TSG file starts with a letter denoting the record type:

- `H` - Header information (including graph definitions)
- `N` - Node definition (exon or transcript segment)
- `E` - Edge definition (splice junction or structural variant)
- `U` - Unordered group (set of elements)
- `P` - Path (ordered traversal through the graph)
- `C` - Chain (alternating nodes and edges)
- `A` - Attribute for any element (metadata)
- `L` - Inter-graph link (connections between different graphs)

### Conceptual Model

In the TSG model:

1. **Graphs (G)** represent independent transcript graphs, each with its own set of nodes and edges.
2. **Chains (C)** are used to build each graph's structure.
3. **Paths (P)** are traversals through the constructed graphs.
4. **Links (L)** establish relationships between elements in different graphs.

This distinction is important: chains define what each graph is, paths define ways to traverse each graph, and links define relationships between graphs.

### Example with Multiple Graphs

```text
# Global headers
H  TSG  1.0
H  reference  GRCh38

# First graph
G  gene_a  name:Z:BRCA1  locus:Z:chr17q21.31

# Nodes for gene_a
N  n1  chr17:+:41196312-41196402  read1:SO,read2:SO  ACGTACGT
N  n2  chr17:+:41199660-41199720  read2:IN,read3:IN  TGCATGCA
N  n3  chr17:+:41203080-41203134  read1:SI,read2:SI  CTGACTGA

# Edges for gene_a
E  e1  n1  n2  chr17,chr17,41196402,41199660,splice
E  e2  n2  n3  chr17,chr17,41199720,41203080,splice

# Chains for gene_a
C  chain1  n1  e1  n2  e2  n3

# Paths for gene_a
P  transcript1  n1+  e1+  n2+  e2+  n3+

# Sets for gene_a
U  exon_set  n1  n2  n3

# Attributes for gene_a elements
A  N  n1  expression:f:10.5
A  N  n1  ptc:i:10
A  P  transcript1  tpm:f:8.2

# Second graph
G  gene_b  name:Z:BRCA2  locus:Z:chr13q13.1

# Nodes for gene_b
N  n1  chr13:+:32315480-32315652  read4:SO,read5:SO  GATTACA
N  n2  chr13:+:32316528-32316800  read4:IN,read5:IN  TACGATCG
N  n3  chr13:+:32319077-32319325  read4:SI,read5:SI  CGTACGTA

# Edges for gene_b
E  e1  n1  n2  chr13,chr13,32315652,32316528,splice
E  e2  n2  n3  chr13,chr13,32316800,32319077,splice

# Chains for gene_b
C  chain1  n1  e1  n2  e2  n3

# Paths for gene_b
P  transcript1  n1+  e1+  n2+  e2+  n3+

# Sets for gene_b
U  exon_set  n1  n2  n3

# Attributes for gene_b elements
A  P  transcript1  tpm:f:3.7

# Inter-graph links (appears after all graph sections)
L  fusion1  gene_a:n3  gene_b:n1  fusion  type:Z:chromosomal
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Apache-2.0](LICENSE)
