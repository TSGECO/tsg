<img src="./doc/logo.png" alt="crates.io" height="100" align="center"/>

# TSG - Transcript Segment Graph

TSG is a Rust library and command-line tool for creating, manipulating, and analyzing transcript segment graphs. It provides a comprehensive framework for modeling segmented transcript data, analyzing non-linear splicing events, and working with genomic structural variants.

## Features

- Parse and write TSG format files
- Build and manipulate transcript segment graphs
- Analyze paths and connectivity between transcript segments
- Support for various element types: nodes, edges, groups, and chains
- Export graphs to DOT format for visualization
- Traverse the graph to identify valid transcript paths
- Read identity tracking to ensure biological validity
- Build graphs from chains and validate path traversals
- Support for genomic coordinates with strand information
- Support for read evidence with types

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

### Creating a Graph Programmatically

```rust
use tsg::graph::{TSGraph, NodeData, EdgeData, StructuralVariant};
use bstr::BString;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut graph = TSGraph::new();

    // Add nodes
    let node1 = NodeData {
        id: "node1".into(),
        reference_id: "chr1".into(),
        ..Default::default()
    };

    let node2 = NodeData {
        id: "node2".into(),
        reference_id: "chr1".into(),
        ..Default::default()
    };

    graph.add_node(node1)?;
    graph.add_node(node2)?;

    // Add an edge between nodes
    let edge = EdgeData {
        id: "edge1".into(),
        ..Default::default()
    };

    graph.add_edge("node1".into(), "node2".into(), edge)?;

    // Write to file
    graph.write_to_file("new_graph.tsg")?;

    Ok(())
}
```

### Building a Graph from Chains

```rust
use tsg::graph::{TSGraph, Group};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create chains
    let chains = vec![
        Group::Chain {
            id: "chain1".into(),
            elements: vec!["n1".into(), "e1".into(), "n2".into()],
            attributes: HashMap::new(),
        },
        Group::Chain {
            id: "chain2".into(),
            elements: vec!["n2".into(), "e2".into(), "n3".into()],
            attributes: HashMap::new(),
        },
    ];

    // Build graph from chains
    let graph = TSGraph::from_chains(chains)?;

    // Write to file
    graph.write_to_file("output.tsg")?;

    Ok(())
}
```

### Finding Valid Paths Through the Graph

```rust
use tsg::graph::TSGraph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let graph = TSGraph::from_file("transcript.tsg")?;

    // Find all valid paths through the graph
    let paths = graph.traverse()?;

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
tsg-cli --help

# Parse and validate a TSG file
tsg-cli validate path/to/file.tsg

# Convert a TSG file to DOT format for visualization
tsg-cli dot path/to/file.tsg > graph.dot

# Extract statistics from a TSG file
tsg-cli stats path/to/file.tsg

# Find all paths through the graph
tsg-cli paths path/to/file.tsg
```

## TSG File Format

The TSG format is a tab-delimited text format representing transcript assemblies as graphs.

### Record Types

Each line in a TSG file starts with a letter denoting the record type:

- `H` - Header information
- `N` - Node definition (exon or transcript segment)
- `E` - Edge definition (splice junction or structural variant)
- `U` - Unordered group (set of elements)
- `O` - Ordered group (path through the graph)
- `C` - Chain (alternating nodes and edges)
- `A` - Attribute for any element (metadata)

### Conceptual Model

In the TSG model:

1. **Chains (C)** are used to build the graph structure. They define the nodes and edges that make up the graph.
2. **Paths (O)** are traversals through the constructed graph.
3. The complete TSG is built by combining all nodes and edges from all chains.
4. After constructing the graph from chains, paths can be defined to represent ways of traversing the graph.

This distinction is important: chains define what the graph is, while paths define ways to traverse the graph.

### Example

```text
# Header information
H  TSG  1.0
H  reference  GRCh38

# Nodes (exons)
N  n1  chr1:+:1000-1200,1500-1700  read1:SO,read2:SO  ACGTACGT
N  n2  chr1:+:2000-2200  read4:SO,read5:SO  TGCATGCA
N  n3  chr1:+:2500-2700  read1:IN,read2:IN,read3:IN,read4:IN  CTGACTGA

# Edges (splice junctions)
E  e1  n1  n2  chr1,chr1,1700,2000,splice
E  e2  n2  n3  chr1,chr1,2200,2500,splice

# Chains (building the graph)
C  chain1  n1 e1 n2 e2 n3

# Paths (traversals)
O  transcript1  n1+ e1+ n2+ e2+ n3+

# Sets (grouping elements)
U  exon_set  n1 n2 n3

# Attributes (metadata)
A  N  n1  expression:f:10.5
A  O  transcript1  tpm:f:8.2
```

### Node Format

Nodes represent exons or transcript segments with the format:

```text
N  <id>  <genomic_location>  <reads>  [<seq>]
```

Where:

- `genomic_location` is in format `chromosome:strand:coordinates` (e.g., `chr1:+:1000-1200,1500-1700`)
- `reads` is a comma-separated list of read IDs with types (e.g., `read1:SO,read2:IN`)
- Read types include:
  - `SO`: Source Node
  - `IN`: Intermediary Node
  - `SI`: Sink Node

### Edge Format

Edges represent splice junctions or structural variants:

```text
E  <id>  <source_id>  <sink_id>  <SV>
```

Where:

- `SV` is in format `reference_name1,reference_name2,breakpoint1,breakpoint2,sv_type`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Apache-2.0](LICENSE)
