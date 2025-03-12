# Transcript Segment Graph (TSG) Format Specification

## Overview

The Transcript Segment Graph (TSG) format is designed for representing transcript assemblies and splicing relationships, adapting concepts from the Graphical Fragment Assembly (GFA) 2.0 specification. TSG allows for the representation of exons, splice junctions, isoforms, and structural variants in a graph-based format. The format supports multiple graphs within a single file.

## Conceptual Model

In the TSG model:

1. **Graphs (G)** represent independent transcript graphs, each with its own set of nodes and edges.
2. **Chains (C)** are used to build the graph structure. They define the nodes and edges that make up the graph.
3. **Paths (P)** are traversals through the constructed graph.
4. The complete TSG is built by combining all nodes and edges from all chains within each graph.
5. After constructing the graph from chains, paths can be defined to represent ways of traversing the graph.

## File Format

TSG is a tab-delimited text format with each line beginning with a single letter that defines the record type. Fields within each line are separated by tabs, and each line represents a distinct element of the transcript graph.

## Multi-Graph Support

TSG supports multiple graphs within a single file using a graph namespace approach. Each element in the file can be associated with a specific graph using a graph ID prefix:

```text
graph_id:element_id
```

For example, `gene_a:n1` refers to node n1 in the graph identified as "gene_a".

## Record Types

### Header (H)

Contains metadata about the file.

```text
H  <tag>  <value>
```

Fields:

- `tag`: Identifier for the header entry
- `value`: Header value

For defining graphs:

```text
H  graph  <graph_id>  [<description>]
```

Fields:

- `graph`: Fixed tag indicating a graph definition
- `graph_id`: Unique identifier for the graph
- `description` (optional): Description of the graph

### Nodes (N)

Represent exons or transcript segments.

```text
N  <graph_id:id>  <genomic_location>  <reads>  [<seq>]
```

Fields:

- `graph_id:id`: Graph-qualified unique identifier for the node
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
E  <graph_id:id>  <graph_id:source_id>  <graph_id:sink_id>  <SV>
```

Fields:

- `graph_id:id`: Graph-qualified unique identifier for the edge
- `graph_id:source_id`: Graph-qualified ID of the source node
- `graph_id:sink_id`: Graph-qualified ID of the target node
- `SV`: Structural variant information in format "reference_name1,reference_name2,breakpoint1,breakpoint2,sv_type"

### Unordered Groups/Sets (U)

Represent unordered collections of graph elements.

```text
U  <graph_id:group_id>  <graph_id:element_id_1> <graph_id:element_id_2> ... <graph_id:element_id_n>
```

Fields:

- `graph_id:group_id`: Graph-qualified unique identifier for the unordered group
- `graph_id:element_id_*`: Space-separated list of graph-qualified element identifiers (nodes, edges, or other groups)

### Ordered Groups/Paths (P)

Represent ordered collections of graph elements where orientation matters.

```text
P  <graph_id:group_id>  <graph_id:oriented_element_id_1> <graph_id:oriented_element_id_2> ... <graph_id:oriented_element_id_n>
```

Fields:

- `graph_id:group_id`: Graph-qualified unique identifier for the ordered group
- `graph_id:oriented_element_id_*`: Space-separated list of graph-qualified element identifiers with orientation (+ or -)

### Chains (C)

Represent explicit paths through the graph with alternating nodes and edges.

```text
C  <graph_id:chain_id>  <graph_id:node_id_1> <graph_id:edge_id_1> <graph_id:node_id_2> <graph_id:edge_id_2> ... <graph_id:node_id_n>
```

Fields:

- `graph_id:chain_id`: Graph-qualified unique identifier for the chain
- Elements: Space-separated list of graph-qualified alternating node and edge identifiers
  - Must start and end with node identifiers
  - Must have an odd number of elements
  - Adjacent elements must be connected in the graph

### Attributes (A)

Optional metadata attached to other elements.

```text
A  <element_type>  <graph_id:element_id>  <tag>:<type>:<value>
```

Fields:

- `element_type`: Type of element (N, E, U, P, or C)
- `graph_id:element_id`: Graph-qualified identifier of the element to attach the attribute to
- `tag`: Name of the attribute
- `type`: Single letter code for the attribute data type
- `value`: Value of the attribute

### Inter-Graph Links (L)

Represents connections between elements in different graphs.

```text
L  <id>  <graph_id1:element_id1>  <graph_id2:element_id2>  <link_type>  [<attributes>]
```

Fields:

- `id`: Unique identifier for the link
- `graph_id1:element_id1`: Graph-qualified identifier for the first element
- `graph_id2:element_id2`: Graph-qualified identifier for the second element
- `link_type`: Type of link (e.g., "fusion", "reference", "containment")
- `attributes` (optional): Comma-separated list of attributes in key:value format

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

### Graph Namespace

1. **Independent Graphs**: Each graph defined with the graph header (H graph graph_id) represents an independent transcript segment graph.
2. **Element Qualification**: Each element ID is qualified with its graph ID using the format `graph_id:element_id`.
3. **Scope**: Elements from one graph can only connect to other elements within the same graph, except through explicit inter-graph links.
4. **Unique Identification**: The graph_id prefix ensures that element IDs are unique across the entire file, even if the same local ID appears in multiple graphs.

### Chains vs. Paths

Chains and paths serve fundamentally different purposes in the TSG format:

1. **Chains as Graph Construction Elements**:

   - Chains (C) are used to build the TSG graph itself.
   - Each chain contributes nodes and edges to the graph structure.
   - The complete graph is constructed by collecting all nodes and edges from all chains.
   - Chains represent the source evidence (e.g., transcript sequences) from which the graph was built.

2. **Paths as Graph Traversals**:
   - Paths (P) are traversals through the already-constructed graph.
   - Paths don't add any new structural elements to the graph.
   - They represent ways of traveling through the existing nodes and edges.
   - Paths can represent transcript isoforms, alternative splicing patterns, or other biological features.

This distinction is critical: chains define what the graph is, while paths define ways to traverse the graph.

### Inter-Graph Links

Inter-graph links provide a way to represent relationships between elements in different graphs:

1. **Types of Links**:

   - **Fusion**: Represents a fusion between transcripts in different graphs
   - **Reference**: Indicates that one element references another
   - **Containment**: Shows that one element contains or is a superset of another
   - **Identity**: Indicates that elements across graphs are identical

2. **Usage Scenarios**:
   - Connecting fusion transcripts across genes
   - Linking alternative assemblies of the same region
   - Creating hierarchical relationships between graphs
   - Cross-referencing between different transcript annotations

## Processing Model

The typical processing flow for a TSG file depends on whether the graph structure (nodes and edges) already exists:

### Case 1: Nodes and Edges Are Explicitly Defined

If the TSG file contains explicit node (N) and edge (E) records:

1. Identify the graphs defined in the file
2. For each graph, read and create the graph structure directly from its records
3. Chains (C) serve as additional evidence or source information
4. Paths (P) define traversals through the explicitly defined graph
5. Process inter-graph links to establish connections between graphs
6. Validate read continuity by checking for shared read IDs across adjacent nodes in paths

### Case 2: Nodes and Edges Are Not Explicitly Defined

If the TSG file does not contain explicit node and edge records, or contains only partial definitions:

1. Identify the graphs defined in the file
2. For each graph, extract and construct all nodes and edges from chains (C)
3. Build the complete graph structure from these extracted elements
4. Chains provide both the structure and the evidence for the graph
5. Paths (P) define traversals through the graph constructed from chains
6. Process inter-graph links to establish connections between graphs
7. Verify read continuity for all paths by ensuring adjacent nodes share common read support

## Type Definitions for Attributes

- `i`: Integer
- `f`: Float
- `Z`: String
- `J`: JSON
- `H`: Hex string
- `B`: Byte array

## Example

```text
# File header
H  TSG  1.0
H  reference  GRCh38

# Graph definitions
H  graph  gene_a  BRCA1 transcripts
H  graph  gene_b  BRCA2 transcripts

# Nodes for gene_a
N  gene_a:n1  chr17:+:41196312-41196402  read1:SO,read2:SO  ACGTACGT
N  gene_a:n2  chr17:+:41199660-41199720  read2:IN,read3:IN  TGCATGCA
N  gene_a:n3  chr17:+:41203080-41203134  read1:SI,read2:SI  CTGACTGA

# Nodes for gene_b
N  gene_b:n1  chr13:+:32315480-32315652  read4:SO,read5:SO  GATTACA
N  gene_b:n2  chr13:+:32316528-32316800  read4:IN,read5:IN  TACGATCG
N  gene_b:n3  chr13:+:32319077-32319325  read4:SI,read5:SI  CGTACGTA

# Edges for gene_a
E  gene_a:e1  gene_a:n1  gene_a:n2  chr17,chr17,41196402,41199660,splice
E  gene_a:e2  gene_a:n2  gene_a:n3  chr17,chr17,41199720,41203080,splice

# Edges for gene_b
E  gene_b:e1  gene_b:n1  gene_b:n2  chr13,chr13,32315652,32316528,splice
E  gene_b:e2  gene_b:n2  gene_b:n3  chr13,chr13,32316800,32319077,splice

# Chains for gene_a
C  gene_a:chain1  gene_a:n1  gene_a:e1  gene_a:n2  gene_a:e2  gene_a:n3

# Chains for gene_b
C  gene_b:chain1  gene_b:n1  gene_b:e1  gene_b:n2  gene_b:e2  gene_b:n3

# Paths for gene_a
P  gene_a:transcript1  gene_a:n1+  gene_a:e1+  gene_a:n2+  gene_a:e2+  gene_a:n3+

# Paths for gene_b
P  gene_b:transcript1  gene_b:n1+  gene_b:e1+  gene_b:n2+  gene_b:e2+  gene_b:n3+

# Sets for gene_a
U  gene_a:exon_set  gene_a:n1  gene_a:n2  gene_a:n3

# Sets for gene_b
U  gene_b:exon_set  gene_b:n1  gene_b:n2  gene_b:n3

# Inter-graph link (e.g., for a fusion transcript)
L  fusion1  gene_a:n3  gene_b:n1  fusion  type:Z:chromosomal

# Attributes
A  N  gene_a:n1  expression:f:10.5
A  N  gene_a:n1  ptc:i:10
A  P  gene_a:transcript1  tpm:f:8.2
A  P  gene_b:transcript1  tpm:f:3.7
```

In this example:

1. Two graphs (gene_a and gene_b) are defined, each with their own nodes, edges, chains, and paths
2. Each element is qualified with its graph ID (e.g., gene_a:n1)
3. An inter-graph link represents a fusion between the last exon of gene_a and the first exon of gene_b
4. Attributes provide additional information about elements in each graph
5. Read continuity can be verified within each graph independently

## Implementation Considerations

### Graph Namespace Handling

Implementations should:

- Parse the graph ID prefix from each element ID
- Maintain separate data structures for each graph
- Enforce that connections (edges, chains, paths) only exist between elements in the same graph
- Process inter-graph links separately from within-graph connections

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

- All elements are properly qualified with a graph ID
- Chains have an odd number of elements (starting and ending with nodes)
- Adjacent elements in chains are correctly connected in the graph
- Group identifiers are unique across U, P, and C types within each graph
- Paths (P-lines) only reference elements that exist in the same graph
- Inter-graph links only reference elements that exist in their respective graphs

### Chain Processing

- Process chains within each graph independently
- When encountering a chain, extract all nodes and edges and add them to the appropriate graph
- The same node or edge can appear in multiple chains within the same graph
- The structural integrity of each graph is defined by its chains

### Path Processing

- Paths do not add new elements to the graph
- Paths must reference existing nodes and edges within the same graph
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

#### Graphs (G)

- **Genes**: Independent genetic loci
- **Transcription Units**: Coordinated transcriptional regions
- **Alternative Assemblies**: Different representations of the same region

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

#### Inter-Graph Links (L) (maybe support)

- **Fusion Transcripts**: Transcripts spanning multiple genes
- **Reference Relationships**: Cross-references between different transcript annotations
- **Containment Relationships**: Hierarchical organization of transcripts

This separation aligns with how many transcript assembly algorithms work:

1. First, chains of exons and splice junctions are identified from the data
2. Then, potential transcripts are derived by traversing the graph in different ways
3. Finally, relationships between different transcript graphs are established
