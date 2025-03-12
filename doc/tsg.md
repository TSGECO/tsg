# Transcript Segment Graph (TSG) Format Specification

## Overview

The Transcript Segment Graph (TSG) format is designed for representing transcript assemblies and splicing relationships, adapting concepts from the Graphical Fragment Assembly (GFA) 2.0 specification. TSG allows for the representation of exons, splice junctions, isoforms, and structural variants in a graph-based format.

## Conceptual Model

In the TSG model:

1. **Chains (C)** are used to build the graph structure. They define the nodes and edges that make up the graph.
2. **Paths (P)** are traversals through the constructed graph.
3. The complete TSG is built by combining all nodes and edges from all chains.
4. After constructing the graph from chains, paths can be defined to represent ways of traversing the graph.

## File Format

TSG is a tab-delimited text format with each line beginning with a single letter that defines the record type. Fields within each line are separated by tabs, and each line represents a distinct element of the transcript graph.

## Record Types

### Header (H)

Contains metadata about the file.

```text
H  <tag>  <value>
```

Fields:

- `tag`: Identifier for the header entry
- `value`: Header value

### Nodes (N)

Represent exons or transcript segments.

```text
N  <id>  <genomic_location>  <reads>  [<seq>]
```

Fields:

- `id`: Unique identifier for the node
- `genomic_location`: Format `chromosome:strand:coordinates` where:
  - `chromosome`: Chromosome name (e.g., "chr1")
  - `strand`: "+" for forward strand, "-" for reverse strand
  - `coordinates`: Comma-separated list of exon coordinates in "start-end" format
- `reads`: Comma-separated list of reads supporting this node, in format `read_id:type`
  - Types might include SO (spanning), IN (internal), SI (significant), etc.
- `seq` (optional): Sequence of the node

### Edges (E)

Represent connections between nodes, including splice junctions or structural variants.

```text
E  <id>  <source_id>  <sink_id>  <SV>
```

Fields:

- `id`: Unique identifier for the edge
- `source_id`: ID of the source node
- `sink_id`: ID of the target node
- `SV`: Structural variant information in format "reference_name1,reference_name2,breakpoint1,breakpoint2,sv_type"

### Unordered Groups/Sets (U)

Represent unordered collections of graph elements.

```text
U  <group_id>  <element_id_1> <element_id_2> ... <element_id_n>
```

Fields:

- `group_id`: Unique identifier for the unordered group
- `element_id_*`: Space-separated list of element identifiers (nodes, edges, or other groups)

### Ordered Groups/Paths (P)

Represent ordered collections of graph elements where orientation matters.

```text
P  <group_id>  <oriented_element_id_1> <oriented_element_id_2> ... <oriented_element_id_n>
```

Fields:

- `group_id`: Unique identifier for the ordered group
- `oriented_element_id_*`: Space-separated list of element identifiers with orientation (+ or -)

### Chains (C)

Represent explicit paths through the graph with alternating nodes and edges.

```text
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

```text
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

Nodes in TSG represent exons or transcript segments. Each node has a genomic location that includes chromosome, strand, and coordinates. The genomic location specifies where the node is located in the reference genome. Nodes can be supported by different types of read evidence through the `reads` field, and can optionally include the sequence.

### Edge Semantics

Edges in TSG represent connections between nodes, such as splice junctions or structural variants. The `SV` field provides details about the genomic context of the connection, including reference names, breakpoints, and the type of structural variant or splice.

### Read Continuity

Read continuity is a critical concept in TSG that ensures valid traversals through the graph:

1. **Definition**: Read continuity requires that specific patterns of read support exist between connected nodes in a path, depending on node types.

2. **Node Types and Continuity Requirements**:

   - **SO (Source Node)**: Represents the start of a read. No read continuity required with previous nodes.
   - **SI (Sink Node)**: Represents the end of a read. No read continuity required with subsequent nodes.
   - **IN (Intermediary Node)**: Represents an internal segment of a read. Must share at least one read ID with both its previous and next nodes in the path.

3. **Validation**:

   - For IN nodes, implementations must verify that at least one common read ID exists between the current node and both its adjacent nodes.
   - SO and SI nodes provide more flexible continuity, allowing for extended paths without requiring end-to-end read support.
   - A path can be considered valid even if it doesn't have a single read spanning its entire length.

4. **Constraints**:

   - Each IN node in a valid path must maintain read continuity with its adjacent nodes.
   - The specific read type (SO, SI, IN) determines the continuity requirements at each position in the path.

5. **Representation**: The read IDs in each node's `reads` field implicitly define the continuity relationships in the graph, while the read types determine the continuity constraints.

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

## Processing Model

The typical processing flow for a TSG file depends on whether the graph structure (nodes and edges) already exists:

### Case 1: Nodes and Edges Are Explicitly Defined

If the TSG file contains explicit node (N) and edge (E) records:

1. Read and create the graph directly from these records
2. Chains (C) serve as additional evidence or source information
3. Paths (O) define traversals through the explicitly defined graph
4. Validate read continuity by checking for shared read IDs across adjacent nodes in paths

### Case 2: Nodes and Edges Are Not Explicitly Defined

If the TSG file does not contain explicit node and edge records, or contains only partial definitions:

1. Extract and construct all nodes and edges from chains (C)
2. Build the complete graph structure from these extracted elements
3. Chains provide both the structure and the evidence for the graph
4. Paths (O) define traversals through the graph constructed from chains
5. Verify read continuity for all paths by ensuring adjacent nodes share common read support

## Type Definitions for Attributes

- `i`: Integer
- `f`: Float
- `Z`: String
- `J`: JSON
- `H`: Hex string
- `B`: Byte array

## Example

```text
# Header information
H  TSG  1.0
H  reference  GRCh38
# Nodes
N  n1  chr1:+:1000-1200,1500-1700  read1:SO,read2:SO  ACGTACGT
N  n2  chr1:+:2000-2200  read4:SO,read5:SO  TGCATGCA
N  n3  chr1:+:2500-2700  read1:IN,read2:IN,read3:IN,read4:IN  CTGACTGA
N  n4  chr1:-:2500-2700  read1:SI,read2:SI  CTGACTGA
N  n5  chr1:+:2500-2700  read3:SI,read4:SI  CTGACTGA
# Edges
E  e1  n1  n3  chr1,chr1,1700,2000,splice
E  e2  n3  n4  chr1,chr1,1700,2000,splice
E  e3  n2  n3  chr1,chr1,2200,2500,splice
E  e4  n3  n5  chr1,chr1,1700,2500,splice
# Chains (building the graph)
C  chain1  n1  e1  n3  e2  n4
C  chain2  n2  e3  n3  e4  n5
# Paths (traversals through the constructed graph)
P  transcript1  n1+  e1+  n3+  e2+  n4+
P  transcript2  n2+  e3+  n3+  e4+  n5+
# Sets (grouping elements)
U  exon_set  n1  n2  n3
# Attributes (metadata)
A  N  n1  expression:f:10.5
A  N  n1  ptc:i:10
A  O  transcript1  tpm:f:8.2
A  O  transcript2  tpm:f:3.7
```

In this example:

1. The graph contains 5 nodes and 4 edges
2. Two chains (chain1 and chain2) represent evidence used to construct the graph
3. Two paths (transcript1 and transcript2) represent ways to traverse the graph
4. One set (exon_set) groups related nodes
5. Attributes provide additional information about nodes and paths
6. Read continuity can be verified:
   - In path transcript1: n1 has SO reads, n3 has IN reads, and n4 has SI reads
     - n3 (IN) properly shares reads (read1, read2) with both n1 and n4
   - In path transcript2: n2 has SO reads, n3 has IN reads, and n5 has SI reads
     - n3 (IN) properly shares reads (read4) with both n2 and n5
   - This demonstrates valid read continuity as required by the node types

## Implementation Considerations

### Genomic Location Parsing

Implementations should carefully parse the genomic location field, which contains:

- Chromosome (e.g., "chr1")
- Strand ("+" or "-")
- Coordinates (comma-separated list of "start-end" pairs)

These components are separated by colons.

### Read Evidence

Read evidence is recorded with both read identifiers and types. Implementations should:

- Parse the read identifier and read type, separated by a colon
- Support different read types (SO, IN, SI, etc.) as used in the implementation

### Validation Requirements

Implementations should validate that:

- Chains have an odd number of elements (starting and ending with nodes)
- Adjacent elements in chains are correctly connected in the graph
- Group identifiers are unique across U, O, and C types
- Paths (O-lines) only reference elements that exist in the graph

### Chain Processing

- When encountering a chain, implementations should extract all nodes and edges and add them to the graph
- The same node or edge can appear in multiple chains
- The structural integrity of the graph is defined by the chains

### Path Processing

- Paths do not add new elements to the graph
- Paths must reference existing nodes and edges
- Paths can include orientation information (+ or -) for elements

### Read Continuity Verification

When processing TSG files, implementations should:

- Extract read IDs from each node's `reads` field along with their types (SO, SI, IN)
- For each path or chain traversal:
  - For IN nodes: Ensure at least one read ID is shared with both the previous and next nodes
  - For SO nodes: No continuity check required with previous nodes
  - For SI nodes: No continuity check required with subsequent nodes
- Flag paths where IN nodes lack proper read continuity as potentially unsupported by the data
- Recognize that valid paths may be constructed even without end-to-end read support, as long as the IN node continuity requirements are satisfied
- Provide options to filter or annotate paths based on different levels of read continuity stringency

### Biological Interpretation in Transcript Analysis

In the context of transcript analysis, the TSG format elements typically represent:

#### Nodes (N)

- **Exons**: Genomic regions that are transcribed and remain in the mature RNA
- Can include multiple segments (e.g., for complex exon structures)
- Read support indicates which sequencing reads support this exon, with different types of support

#### Edges (E)

- **Splice Junctions**: Connections between exons
- **Structural Variants**: Genomic rearrangements like fusions, deletions, or insertions
- The SV field provides details on the exact genomic coordinates

#### Chains (C)

- **Original Transcripts**: Complete transcript sequences observed in the data
- **Source Evidence**: The actual RNA molecules detected
- **Assembled Transcripts**: Transcripts assembled from read data
- These build the structure of the transcript graph

#### Paths (P)

- **Transcript Isoforms**: Alternative splicing variants
- **Expression Patterns**: Different ways genes are expressed
- **Predicted Transcripts**: Computationally predicted transcript models
- These are ways to traverse the established transcript graph

This separation aligns with how many transcript assembly algorithms work:

1. First, chains of exons and splice junctions are identified from the data
2. Then, potential transcripts are derived by traversing the graph in different ways
