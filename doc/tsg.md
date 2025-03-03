# Transcript Segment Graph (TSG) Format Specification

## Overview

The Transcript Segment Graph (TSG) format is designed for representing transcript assemblies and splicing relationships, adapting concepts from the Graphical Fragment Assembly (GFA) 2.0 specification. TSG allows for the representation of exons, splice junctions, isoforms, and structural variants in a graph-based format.

## Conceptual Model

In the TSG model:

1. **Chains (C)** are used to build the graph structure. They define the nodes and edges that make up the graph.
2. **Paths (O)** are traversals through the constructed graph.
3. The complete TSG is built by combining all nodes and edges from all chains.
4. After constructing the graph from chains, paths can be defined to represent ways of traversing the graph.

## File Format

TSG is a tab-delimited text format with each line beginning with a single letter that defines the record type. Fields within each line are separated by tabs, and each line represents a distinct element of the transcript graph.

## Record Types

### Header (H)

Contains metadata about the file.

```
H  <tag>  <type>  <value>
```

Fields:

- `tag`: Identifier for the header entry
- `type`: Type of the value (optional)
- `value`: Header value

### Nodes (N)

Represent exons or transcript segments.

```
N  <id>  <exons>  <reads>  [<seq>]
```

Fields:

- `id`: Unique identifier for the node
- `exons`: Comma-separated list of start-end coordinates, e.g., "start1-end1,start2-end2"
- `reads`: List of read identifiers supporting this node, comma-separated
- `seq` (optional): Sequence of the node

### Edges (E)

Represent connections between nodes, including splice junctions or structural variants.

```
E  <id>  <source_id>  <sink_id>  <SV>
```

Fields:

- `id`: Unique identifier for the edge
- `source_id`: ID of the source node
- `sink_id`: ID of the target node
- `SV`: Structural variant information in format "reference_name1,reference_name2,breakpoint1,breakpoint2,sv_type"

### Unordered Groups/Sets (U)

Represent unordered collections of graph elements.

```
U  <group_id>  <element_id_1> <element_id_2> ... <element_id_n>
```

Fields:

- `group_id`: Unique identifier for the unordered group
- `element_id_*`: Space-separated list of element identifiers (nodes, edges, or other groups)

### Ordered Groups/Paths (O)

Represent ordered collections of graph elements where orientation matters.

```
O  <group_id>  <oriented_element_id_1> <oriented_element_id_2> ... <oriented_element_id_n>
```

Fields:

- `group_id`: Unique identifier for the ordered group
- `oriented_element_id_*`: Space-separated list of element identifiers with orientation (+ or -)

### Chains (C)

Represent explicit paths through the graph with alternating nodes and edges.

```
C  <chain_id>  <node_id_1> <edge_id_1> <node_id_2> <edge_id_2> ... <node_id_n>
```

Fields:

- `chain_id`: Unique identifier for the chain
- Elements: Space-separated list of alternating node and edge identifiers
  - Must start and end with node identifiers
  - Must have an odd number of elements
  - Adjacent elements must be connected in the graph

### Attributes (A)

Optional metadata attached to other elements.

```
A  <element_type>  <element_id>  <tag>  <type>  <value>
```

Fields:

- `element_type`: Type of element (N, E, U, O, or C)
- `element_id`: Identifier of the element to attach the attribute to
- `tag`: Name of the attribute
- `type`: Single letter code for the attribute data type
- `value`: Value of the attribute

## Semantics

### Node Semantics

Nodes in TSG represent exons or transcript segments. Each node can span multiple genomic regions through the `exons` field, allowing for representation of complex exonic structures. Nodes can be backed by read evidence through the `reads` field, and can optionally include the sequence.

### Edge Semantics

Edges in TSG represent connections between nodes, such as splice junctions or structural variants. The `SV` field provides details about the genomic context of the connection, including reference names, breakpoints, and the type of structural variant or splice.

### Group and Chain Semantics

- **Unordered Groups (U)**: Represent the subgraph induced by the vertices and edges in the collection. Includes all edges between pairs of segments in the list and all segments adjacent to edges in the list.
- **Ordered Groups (O)**: Represent paths in the graph consisting of the listed objects and implied adjacent objects between consecutive items, where orientation matters.
- **Chains (C)**: Represent a concrete path through the graph with explicitly listed nodes and edges in alternating sequence. A chain must start and end with a node, and must maintain the correct connectivity between elements.

### Chains vs. Paths

Chains and paths serve fundamentally different purposes in the TSG format:

1. **Chains as Graph Construction Elements**:

   - Chains (C) are used to build the TSG graph itself.
   - Each chain contributes nodes and edges to the graph structure.
   - The complete graph is constructed by collecting all nodes and edges from all chains.
   - Chains represent the source evidence (e.g., transcript sequences) from which the graph was built.

2. **Paths as Graph Traversals**:
   - Paths (O) are traversals through the already-constructed graph.
   - Paths don't add any new structural elements to the graph.
   - They represent ways of traveling through the existing nodes and edges.
   - Paths can represent transcript isoforms, alternative splicing patterns, or other biological features.

This distinction is critical: chains define what the graph is, while paths define ways to traverse the graph.

### Hierarchical Relationships

- A set (U) can contain references to paths (O) or chains (C), but not vice versa
- U-lines, O-lines, and C-lines must have unique identifiers (cannot share the same name)

## Type Definitions for Attributes

- `i`: Integer
- `f`: Float
- `Z`: String
- `J`: JSON
- `H`: Hex string
- `B`: Byte array

## Example

```
# Header information
H  TSG  1.0
H  reference  GRCh38

# Nodes (exons)
N  n1  1000-1200,1500-1700  read1,read2,read3  ACGTACGT...
N  n2  2000-2200  read4,read5  TGCATGCA...
N  n3  2500-2700  read6,read7  CTGACTGA...

# Edges (splice junctions)
E  e1  n1  n2  chr1,chr1,1700,2000,splice
E  e2  n2  n3  chr1,chr1,2200,2500,splice
E  e3  n1  n3  chr1,chr1,1700,2500,splice

# Chains (building the graph)
C  chain1  n1 e1 n2 e2 n3   # Regular splicing
C  chain2  n1 e3 n3         # Exon skipping event

# Paths (traversals through the constructed graph)
O  transcript1  n1+ e1+ n2+ e2+ n3+
O  transcript2  n1+ e3+ n3+

# Sets (grouping elements)
U  exon_set  n1 n2 n3

# Attributes (metadata)
A  N  n1  expression  f  10.5
A  O  transcript1  tpm  f  8.2
A  O  transcript2  tpm  f  3.7
```

In this example:

1. The graph is constructed from the nodes and edges in two chains (chain1 and chain2)
2. Two paths (transcript1 and transcript2) define ways to traverse the graph
3. Both chains and paths describe the same structures, but chains built the graph while paths traverse it

## Usage Guidelines

### When to Use Unordered Groups (U)

- Representing gene families or clusters
- Collecting exons that belong to the same gene
- Defining subgraphs for analysis or visualization
- Creating sets of related elements without imposing order

### When to Use Ordered Groups/Paths (O)

- Representing traversals through the established graph
- Defining transcript isoforms as paths through the existing graph
- Analyzing potential transcript variants
- Defining traversals where orientation matters (e.g., strand-specific transcription)
- Highlighting specific routes through a complex transcript graph

### When to Use Chains (C)

- Building the fundamental structure of the TSG graph
- Contributing the nodes and edges that make up the graph
- Representing the source transcript evidence used to construct the graph
- Preserving the original transcript observations
- Documenting how the graph was constructed

## Processing Model

The typical processing flow for a TSG file depends on whether the graph structure (nodes and edges) already exists:

### Case 1: Nodes and Edges Are Explicitly Defined

If the TSG file contains explicit node (N) and edge (E) records:

1. Read and create the graph directly from these records
2. Chains (C) serve as additional evidence or source information
3. Paths (O) define traversals through the explicitly defined graph

### Case 2: Nodes and Edges Are Not Explicitly Defined

If the TSG file does not contain explicit node and edge records, or contains only partial definitions:

1. Extract and construct all nodes and edges from chains (C)
2. Build the complete graph structure from these extracted elements
3. Chains provide both the structure and the evidence for the graph
4. Paths (O) define traversals through the graph constructed from chains

This dual approach allows TSG to represent both:

- Explicitly constructed graphs with supporting evidence (Case 1)
- Graphs that are implicitly defined by their source chains (Case 2)

### Performance Considerations

- For large graphs, consider indexing nodes and edges for efficient lookup
- When processing chains, validation of connectivity can be computationally expensive
- Consider lazy loading of sequences for memory efficiency

### Biological Interpretation in Transcript Analysis

In the context of transcript analysis, the TSG format elements typically represent:

### Nodes (N)

- **Exons**: Genomic regions that are transcribed and remain in the mature RNA
- Can include multiple segments (e.g., for complex exon structures)
- Read support indicates which sequencing reads support this exon

### Edges (E)

- **Splice Junctions**: Connections between exons
- **Structural Variants**: Genomic rearrangements like fusions, deletions, or insertions
- The SV field provides details on the exact genomic coordinates

### Chains (C)

- **Original Transcripts**: Complete transcript sequences observed in the data
- **Source Evidence**: The actual RNA molecules detected
- **Assembled Transcripts**: Transcripts assembled from read data
- These build the structure of the transcript graph

### Paths (O)

- **Transcript Isoforms**: Alternative splicing variants
- **Expression Patterns**: Different ways genes are expressed
- **Predicted Transcripts**: Computationally predicted transcript models
- These are ways to traverse the established transcript graph

This separation aligns with how many transcript assembly algorithms work:

1. First, chains of exons and splice junctions are identified from the data
2. Then, potential transcripts are derived by traversing the graph in different ways
