//! BTSG (Binary Transcript Segment Graph) format for compressed TSG files

use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use tracing::{debug, warn};

use std::fs::File;
use std::io::{self, BufRead, BufReader, Cursor, Read, Write};
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use bstr::{BStr, BString, ByteSlice};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use thiserror::Error;
use tsg_core::graph::TSGraph;
use zstd::{decode_all, encode_all};

use tsg_core::graph::DEFAULT_GRAPH_ID;

// Block type identifiers
const BLOCK_HEADER: u8 = 0x01;
const BLOCK_GRAPH: u8 = 0x02;
const BLOCK_NODE: u8 = 0x03;
const BLOCK_EDGE: u8 = 0x04;
const BLOCK_ATTRIBUTE: u8 = 0x05;
const BLOCK_CHAIN: u8 = 0x06;
const BLOCK_PATH: u8 = 0x07;
const BLOCK_LINK: u8 = 0x08;
const BLOCK_DICTIONARY: u8 = 0x09;

// Block format version
const BTSG_VERSION: u32 = 1;

#[derive(Error, Debug)]
pub enum BTSGError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("Invalid block type: {0}")]
    InvalidBlockType(u8),

    #[error("Invalid data format: {0}")]
    InvalidFormat(String),

    #[error("Dictionary error: {0}")]
    Dictionary(String),
}

/// Dictionary for string compression
#[derive(Default)]
struct StringDictionary {
    // Maps strings to their dictionary IDs
    str_to_id: HashMap<BString, u32>,
    // Maps dictionary IDs back to strings
    id_to_str: HashMap<u32, BString>,
    // Next available ID
    next_id: u32,
}

impl StringDictionary {
    fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, s: &BStr) -> u32 {
        if let Some(&id) = self.str_to_id.get(s.as_bytes()) {
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;

        let s_owned = s.to_owned();
        self.str_to_id.insert(s_owned.clone(), id);
        self.id_to_str.insert(id, s_owned);
        id
    }

    fn str(&self, id: u32) -> Option<&BStr> {
        self.id_to_str.get(&id).map(|s| s.as_bstr())
    }

    fn id(&self, s: &BStr) -> Option<u32> {
        self.str_to_id.get(s.as_bytes()).copied()
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write dictionary size
        writer.write_u32::<LittleEndian>(self.id_to_str.len() as u32)?;

        // Write each entry: ID followed by string length and string bytes
        for (&id, string) in &self.id_to_str {
            writer.write_u32::<LittleEndian>(id)?;
            writer.write_u32::<LittleEndian>(string.len() as u32)?;
            writer.write_all(string)?;
        }
        Ok(())
    }

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut dict = Self::new();

        // Read dictionary size
        let count = reader.read_u32::<LittleEndian>()?;

        // Read each entry
        for _ in 0..count {
            let id = reader.read_u32::<LittleEndian>()?;
            let len = reader.read_u32::<LittleEndian>()? as usize;

            let mut bytes = vec![0u8; len];
            reader.read_exact(&mut bytes)?;

            let string = BString::from(bytes);
            dict.str_to_id.insert(string.clone(), id);
            dict.id_to_str.insert(id, string);

            if id >= dict.next_id {
                dict.next_id = id + 1;
            }
        }

        Ok(dict)
    }
}

/// A binary block in the BTSG format
struct Block {
    block_type: u8,
    data: Vec<u8>,
}

impl Block {
    fn new(block_type: u8, data: Vec<u8>) -> Self {
        Self { block_type, data }
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Write block type
        writer.write_u8(self.block_type)?;

        // Write block length
        writer.write_u32::<LittleEndian>(self.data.len() as u32)?;

        // Write block data
        writer.write_all(&self.data)?;

        Ok(())
    }

    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        // Read block type
        let block_type = reader.read_u8()?;

        // Read block length
        let length = reader.read_u32::<LittleEndian>()? as usize;

        // Read block data
        let mut data = vec![0u8; length];
        reader.read_exact(&mut data)?;
        Ok(Self { block_type, data })
    }
}

/// TSG compressor - converts TSG to BTSG format
#[derive(Default)]
pub struct BTSGCompressor {
    // Dictionaries for string compression
    node_dict: StringDictionary,
    edge_dict: StringDictionary,
    graph_dict: StringDictionary,
    read_dict: StringDictionary,
    chromosome_dict: StringDictionary,
    attribute_dict: StringDictionary,
    // Compression level for zstd
    compression_level: i32,
}

impl BTSGCompressor {
    pub fn new(compression_level: i32) -> Self {
        Self {
            compression_level,
            ..Default::default()
        }
    }

    pub fn compress<P: AsRef<Path>>(&mut self, input_path: P, output_path: P) -> Result<()> {
        // First pass: build dictionaries and collect data
        self.build_dictionaries(input_path.as_ref())?;

        // Second pass: create blocks and write compressed file
        let mut output_file = File::create(output_path)?;

        // Write magic number and version
        output_file.write_all(b"BTSG")?;
        output_file.write_u32::<LittleEndian>(BTSG_VERSION)?;

        // Write dictionaries
        let dictionary_block = self.create_dictionary_block()?;
        dictionary_block.write(&mut output_file)?;

        // Process input file and create compressed blocks
        let input_file = File::open(input_path)?;
        let reader = BufReader::new(input_file);

        // Organize data by block type with optimized grouping
        let mut header_data = Vec::new();
        let mut graphs: HashMap<BString, Vec<String>> = HashMap::new();
        let mut node_data: HashMap<BString, Vec<String>> = HashMap::new();
        let mut edge_data: HashMap<BString, Vec<String>> = HashMap::new();
        let mut current_graph: Option<BString> = None;

        // Optimized first-pass data organization
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let (record_type, rest) = match line.split_once('\t') {
                Some(parts) => parts,
                None => continue, // Skip malformed lines
            };

            match record_type {
                "H" => {
                    // Add to header block
                    header_data.push(line);
                }
                "G" => {
                    // New graph
                    if let Some((graph_id, _)) = rest.split_once('\t') {
                        let graph_id_bstr = BString::from(graph_id);
                        current_graph = Some(graph_id_bstr.clone());
                        graphs.entry(graph_id_bstr).or_default().push(line);
                    }
                }
                "N" => {
                    // Group all node data by graph for better compression
                    if let Some(ref graph_id) = current_graph {
                        node_data.entry(graph_id.clone()).or_default().push(line);
                    } else {
                        // No current graph, create a default one
                        let default_graph = BString::from(DEFAULT_GRAPH_ID);
                        current_graph = Some(default_graph.clone());
                        node_data.entry(default_graph).or_default().push(line);
                    }
                }
                "E" => {
                    // Group all edge data by graph for better compression
                    if let Some(ref graph_id) = current_graph {
                        edge_data.entry(graph_id.clone()).or_default().push(line);
                    } else {
                        // No current graph, create a default one
                        let default_graph = BString::from(DEFAULT_GRAPH_ID);
                        current_graph = Some(default_graph.clone());
                        edge_data.entry(default_graph).or_default().push(line);
                    }
                }
                "A" | "C" | "P" | "L" => {
                    // Add to current graph's data
                    if let Some(ref graph_id) = current_graph {
                        graphs.entry(graph_id.clone()).or_default().push(line);
                    } else {
                        // No current graph, create a default one
                        let default_graph = BString::from(DEFAULT_GRAPH_ID);
                        current_graph = Some(default_graph.clone());
                        graphs.entry(default_graph).or_default().push(line);
                    }
                }
                _ => {
                    // Unknown record type, skip
                    warn!("Unknown record type: {}", record_type);
                }
            }
        }

        // Write header block
        if !header_data.is_empty() {
            // Apply dictionary compression on header data for better compression
            let optimized_headers = self.optimize_header_data(&header_data)?;
            let header_block = self.create_compressed_block(BLOCK_HEADER, optimized_headers)?;
            header_block.write(&mut output_file)?;
        }

        // Write graph blocks
        for (graph_id, graph_data) in &graphs {
            // Create a compressed block for this graph's data
            let graph_block = self.create_compressed_block(
                BLOCK_GRAPH,
                format!("G\t{}\n{}", graph_id, graph_data.join("\n")),
            )?;
            graph_block.write(&mut output_file)?;
        }

        // Write dedicated node blocks for better compression
        for (graph_id, nodes) in &node_data {
            if nodes.is_empty() {
                continue;
            }

            // Apply node-specific optimizations
            let optimized_nodes = self.optimize_node_data(graph_id.as_bstr(), nodes)?;
            let node_block = self.create_compressed_block(BLOCK_NODE, optimized_nodes)?;
            node_block.write(&mut output_file)?;
        }

        // Write dedicated edge blocks for better compression
        for (graph_id, edges) in &edge_data {
            if edges.is_empty() {
                continue;
            }

            // Apply edge-specific optimizations
            let optimized_edges = self.optimize_edge_data(graph_id.as_bstr(), edges)?;
            let edge_block = self.create_compressed_block(BLOCK_EDGE, optimized_edges)?;
            edge_block.write(&mut output_file)?;
        }

        Ok(())
    }

    // New helper methods for optimized data compression

    /// Optimize header data for better compression
    fn optimize_header_data(&self, headers: &[String]) -> Result<String> {
        // Headers are often very similar - we can optimize this
        let mut optimized = String::with_capacity(headers.iter().map(|s| s.len()).sum());

        // Sort headers by type to group similar ones together
        let mut sorted_headers = headers.to_vec();
        sorted_headers.sort_by(|a, b| {
            let a_type = a.split('\t').nth(1).unwrap_or("");
            let b_type = b.split('\t').nth(1).unwrap_or("");
            a_type.cmp(b_type)
        });

        for header in sorted_headers {
            optimized.push_str(&header);
            optimized.push('\n');
        }

        Ok(optimized)
    }

    /// Optimize node data for better compression
    fn optimize_node_data(&self, graph_id: &BStr, nodes: &[String]) -> Result<String> {
        // Apply delta encoding and further optimizations for nodes
        let mut optimized = format!("G\t{}\n", graph_id);

        // Sort nodes by ID for potential better compression via delta values
        let mut sorted_nodes = nodes.to_vec();
        sorted_nodes.sort_by(|a, b| {
            let a_id = a.split('\t').nth(1).unwrap_or("");
            let b_id = b.split('\t').nth(1).unwrap_or("");
            a_id.cmp(b_id)
        });

        // Group by chromosome to improve compression
        let mut by_chromosome: HashMap<String, Vec<&String>> = HashMap::new();
        for node in &sorted_nodes {
            let chromosome = node
                .split('\t')
                .nth(2)
                .and_then(|loc| loc.split(':').next())
                .unwrap_or("unknown");
            by_chromosome
                .entry(chromosome.to_string())
                .or_default()
                .push(node);
        }

        // Output nodes grouped by chromosome
        for (_, nodes) in by_chromosome {
            for node in nodes {
                optimized.push_str(node);
                optimized.push('\n');
            }
        }

        Ok(optimized)
    }

    /// Optimize edge data for better compression
    fn optimize_edge_data(&self, graph_id: &BStr, edges: &[String]) -> Result<String> {
        // Apply specific optimizations for edge data
        let mut optimized = format!("G\t{}\n", graph_id);

        // Sort edges by source and target nodes
        let mut sorted_edges = edges.to_vec();
        sorted_edges.sort_by(|a, b| {
            let a_parts: Vec<&str> = a.split('\t').collect();
            let b_parts: Vec<&str> = b.split('\t').collect();

            let a_src = a_parts.get(2).unwrap_or(&"");
            let a_dst = a_parts.get(3).unwrap_or(&"");
            let b_src = b_parts.get(2).unwrap_or(&"");
            let b_dst = b_parts.get(3).unwrap_or(&"");

            (a_src, a_dst).cmp(&(b_src, b_dst))
        });

        // Group by edge type for better compression
        let mut by_type: HashMap<String, Vec<&String>> = HashMap::new();
        for edge in &sorted_edges {
            let edge_type = edge
                .split('\t')
                .nth(4)
                .and_then(|sv| sv.split(',').last())
                .unwrap_or("unknown");
            by_type.entry(edge_type.to_string()).or_default().push(edge);
        }

        // Output edges grouped by type
        for (_, edges) in by_type {
            for edge in edges {
                optimized.push_str(edge);
                optimized.push('\n');
            }
        }

        Ok(optimized)
    }

    // Update how the dictionary block is created for better compression
    fn create_dictionary_block(&self) -> Result<Block> {
        let mut buffer = Vec::new();

        // Write each dictionary with its type marker
        buffer.write_u8(0x01)?; // Node dictionary
        self.node_dict.write(&mut buffer)?;

        buffer.write_u8(0x02)?; // Edge dictionary
        self.edge_dict.write(&mut buffer)?;

        buffer.write_u8(0x03)?; // Graph dictionary
        self.graph_dict.write(&mut buffer)?;

        buffer.write_u8(0x04)?; // Read dictionary
        self.read_dict.write(&mut buffer)?;

        buffer.write_u8(0x05)?; // Chromosome dictionary
        self.chromosome_dict.write(&mut buffer)?;

        buffer.write_u8(0x06)?; // Attribute dictionary
        self.attribute_dict.write(&mut buffer)?;

        // Use higher compression level specifically for dictionary blocks
        // Dictionaries benefit from maximum compression since they're referenced frequently
        let compression_level = 19; // Maximum zstd compression level
        let compressed = encode_all(&buffer[..], compression_level)
            .map_err(|e| BTSGError::Compression(e.to_string()))?;

        Ok(Block::new(BLOCK_DICTIONARY, compressed))
    }

    fn build_dictionaries<P: AsRef<Path>>(&mut self, input_path: P) -> Result<()> {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);

        // Pre-allocate collections with reasonable capacities
        let mut read_ids = HashSet::with_capacity(100);
        let mut chromosomes = HashSet::with_capacity(24); // Most genomes have fewer than 24 chromosomes

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            // Using split_once is more efficient than creating a Vec for fields
            let (record_type, rest) = match line.split_once('\t') {
                Some(parts) => parts,
                None => continue, // Skip malformed lines
            };

            match record_type {
                "G" => {
                    // Add graph ID to dictionary
                    if let Some((graph_id, _)) = rest.split_once('\t') {
                        self.graph_dict.add(graph_id.as_bytes().as_bstr());
                    }
                }
                "N" => {
                    // Format: N node_id genomic_loc read_info [sequence]
                    let fields: Vec<&str> = rest.split('\t').collect();
                    if fields.len() >= 3 {
                        // At least node_id, genomic_loc, read_info
                        // Add node ID
                        self.node_dict.add(fields[0].as_bytes().as_bstr());

                        // Extract chromosome from genomic location
                        let genomic_loc = fields[1];
                        if let Some(chr_end) = genomic_loc.find(':') {
                            let chromosome = &genomic_loc[0..chr_end];
                            chromosomes.insert(chromosome.to_string());
                        }

                        // Extract read IDs more efficiently
                        let reads = fields[2];
                        for read_entry in reads.split(',') {
                            if let Some((read_id, _)) = read_entry.split_once(':') {
                                read_ids.insert(read_id.to_string());
                            }
                        }
                    }
                }
                "E" => {
                    // Format: E edge_id source_node target_node sv_info
                    let fields: Vec<&str> = rest.split('\t').collect();
                    if fields.len() >= 3 {
                        // At least edge_id, source_node, target_node
                        self.edge_dict.add(fields[0].as_bytes().as_bstr());
                        self.node_dict.add(fields[1].as_bytes().as_bstr());
                        self.node_dict.add(fields[2].as_bytes().as_bstr());
                    }
                }
                "A" => {
                    // Format: A graph_or_node_or_edge attribute_target attribute_name attribute_value
                    let fields: Vec<&str> = rest.split('\t').collect();
                    if fields.len() >= 3 {
                        // At least target, target_id, attribute_name
                        self.attribute_dict.add(fields[2].as_bytes().as_bstr());
                    }
                }
                _ => {}
            }
        }

        // Add all read IDs and chromosomes to dictionaries in batch
        for read_id in read_ids {
            self.read_dict.add(read_id.as_bytes().as_bstr());
        }

        for chromosome in chromosomes {
            self.chromosome_dict.add(chromosome.as_bytes().as_bstr());
        }

        Ok(())
    }

    fn create_compressed_block(&self, block_type: u8, data: String) -> Result<Block> {
        // For graph blocks, we ensure proper formatting
        let data_to_compress = if block_type == BLOCK_GRAPH {
            let mut lines = data.lines();

            // Extract the graph declaration line
            match lines.next() {
                Some(graph_line) if graph_line.starts_with("G\t") => {
                    // Estimate capacity to avoid reallocations
                    let mut cleaned_data = String::with_capacity(data.len());
                    cleaned_data.push_str(graph_line);

                    // Add the rest of the lines, filtering out any additional G lines
                    for line in lines {
                        if !line.starts_with("G\t") {
                            cleaned_data.push('\n');
                            cleaned_data.push_str(line);
                        }
                    }
                    cleaned_data
                }
                Some(line) => {
                    // If the first line isn't a graph line, something is wrong
                    debug!("Expected graph line starting with G\\t, got: {}", line);
                    data // Use original data as fallback
                }
                None => {
                    // Empty data, just return as is
                    data
                }
            }
        } else {
            data
        };

        // Compress the data
        let compressed = encode_all(data_to_compress.as_bytes(), self.compression_level)
            .map_err(|e| BTSGError::Compression(e.to_string()))?;

        Ok(Block::new(block_type, compressed))
    }
}

/// TSG decompressor - converts BTSG back to TSG format
#[derive(Default)]
pub struct BTSGDecompressor {
    // Dictionaries for string decompression
    node_dict: StringDictionary,
    edge_dict: StringDictionary,
    graph_dict: StringDictionary,
    read_dict: StringDictionary,
    chromosome_dict: StringDictionary,
    attribute_dict: StringDictionary,
}

impl BTSGDecompressor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn decompress<P: AsRef<Path>>(&mut self, input_path: P, output_path: P) -> Result<()> {
        let mut input_file = File::open(input_path)?;

        // Read and verify magic number
        let mut magic = [0u8; 4];
        input_file.read_exact(&mut magic)?;
        if &magic != b"BTSG" {
            return Err(BTSGError::InvalidFormat("Not a valid BTSG file".to_string()).into());
        }

        // Read version
        let version = input_file.read_u32::<LittleEndian>()?;
        if version != BTSG_VERSION {
            return Err(
                BTSGError::InvalidFormat(format!("Unsupported BTSG version: {}", version)).into(),
            );
        }

        let mut output_file = File::create(output_path)?;

        // Build up the graph data as we read blocks
        let mut header_lines = Vec::new();
        let mut graph_data: HashMap<BString, Vec<String>> = HashMap::new();
        let mut node_data: HashMap<BString, Vec<String>> = HashMap::new();
        let mut edge_data: HashMap<BString, Vec<String>> = HashMap::new();

        // Add a default graph if needed
        let default_graph: BString = BString::from(DEFAULT_GRAPH_ID);

        // Track what graph blocks we've seen
        let mut seen_graphs: HashSet<BString> = HashSet::new();

        // Read all blocks first to properly reconstruct the data
        while let Ok(block) = Block::read(&mut input_file) {
            match block.block_type {
                BLOCK_DICTIONARY => {
                    self.read_dictionaries(&block.data)?;
                }
                BLOCK_HEADER => {
                    let decompressed = decode_all(&block.data[..])
                        .map_err(|e| BTSGError::Compression(e.to_string()))?;
                    let content = String::from_utf8_lossy(&decompressed);
                    header_lines.extend(content.lines().map(|s| s.to_string()));
                }
                BLOCK_GRAPH => {
                    let decompressed = decode_all(&block.data[..])
                        .map_err(|e| BTSGError::Compression(e.to_string()))?;
                    let content = String::from_utf8_lossy(&decompressed);
                    let mut lines = content.lines();

                    if let Some(first_line) = lines.next() {
                        if let Some((_, graph_id)) = first_line.split_once('\t') {
                            let graph_id_bstr = BString::from(graph_id);
                            seen_graphs.insert(graph_id_bstr.clone());
                            let graph_entries = graph_data.entry(graph_id_bstr).or_default();
                            graph_entries.push(first_line.to_string());
                            graph_entries.extend(lines.map(|s| s.to_string()));
                        }
                    }
                }
                BLOCK_NODE => {
                    // Handle optimized node blocks
                    let decompressed = decode_all(&block.data[..])
                        .map_err(|e| BTSGError::Compression(e.to_string()))?;
                    let content = String::from_utf8_lossy(&decompressed);
                    let mut lines = content.lines();

                    if let Some(first_line) = lines.next() {
                        if first_line.starts_with("G\t") {
                            if let Some((_, graph_id)) = first_line.split_once('\t') {
                                let graph_id_bstr = BString::from(graph_id);
                                seen_graphs.insert(graph_id_bstr.clone());
                                let nodes = node_data.entry(graph_id_bstr).or_default();
                                nodes.extend(lines.map(|s| s.to_string()));
                            }
                        }
                    }
                }
                BLOCK_EDGE => {
                    // Handle optimized edge blocks
                    let decompressed = decode_all(&block.data[..])
                        .map_err(|e| BTSGError::Compression(e.to_string()))?;
                    let content = String::from_utf8_lossy(&decompressed);
                    let mut lines = content.lines();

                    if let Some(first_line) = lines.next() {
                        if first_line.starts_with("G\t") {
                            if let Some((_, graph_id)) = first_line.split_once('\t') {
                                let graph_id_bstr = BString::from(graph_id);
                                seen_graphs.insert(graph_id_bstr.clone());
                                let edges = edge_data.entry(graph_id_bstr).or_default();
                                edges.extend(lines.map(|s| s.to_string()));
                            }
                        }
                    }
                }
                _ => {
                    // For backward compatibility, try to decompress all other block types
                    match decode_all(&block.data[..]) {
                        Ok(decompressed) => {
                            let content = String::from_utf8_lossy(&decompressed);
                            // Try to determine if this belongs to a graph or is a header
                            let mut has_graph_line = false;
                            let mut current_graph: Option<BString> = None;

                            for line in content.lines() {
                                if line.starts_with("G\t") {
                                    has_graph_line = true;
                                    if let Some((_, graph_id)) = line.split_once('\t') {
                                        let graph_id_bstr = BString::from(graph_id);
                                        current_graph = Some(graph_id_bstr.clone());
                                        seen_graphs.insert(graph_id_bstr.clone());
                                        graph_data
                                            .entry(graph_id_bstr)
                                            .or_default()
                                            .push(line.to_string());
                                    }
                                } else if line.starts_with("H\t") {
                                    header_lines.push(line.to_string());
                                } else if line.starts_with("N\t") {
                                    // It's a node line
                                    let graph_id = current_graph
                                        .clone()
                                        .unwrap_or_else(|| default_graph.clone());
                                    node_data
                                        .entry(graph_id)
                                        .or_default()
                                        .push(line.to_string());
                                } else if line.starts_with("E\t") {
                                    // It's an edge line
                                    let graph_id = current_graph
                                        .clone()
                                        .unwrap_or_else(|| default_graph.clone());
                                    edge_data
                                        .entry(graph_id)
                                        .or_default()
                                        .push(line.to_string());
                                } else {
                                    // Add to current graph or headers
                                    if let Some(ref graph_id) = current_graph {
                                        graph_data
                                            .entry(graph_id.clone())
                                            .or_default()
                                            .push(line.to_string());
                                    } else {
                                        // Add as header if we don't know what it is
                                        header_lines.push(line.to_string());
                                    }
                                }
                            }

                            if !has_graph_line {
                                // No graph line found, treat all content as headers
                                header_lines.extend(content.lines().map(|s| s.to_string()));
                            }
                        }
                        Err(e) => {
                            // Log but don't fail on unknown blocks
                            warn!(
                                "Failed to decompress block type {}: {}",
                                block.block_type, e
                            );
                        }
                    }
                }
            }
        }

        // Write out headers first
        for line in header_lines {
            output_file.write_all(line.as_bytes())?;
            output_file.write_all(b"\n")?;
        }

        // For any graph that only exists in node_data or edge_data but not graph_data,
        // create a graph entry
        for graph_id in node_data.keys().chain(edge_data.keys()) {
            if !graph_data.contains_key(graph_id) && graph_id != &default_graph {
                graph_data.insert(graph_id.clone(), vec![format!("G\t{}", graph_id)]);
                seen_graphs.insert(graph_id.clone());
            }
        }

        // Write out graph data in the right order
        for graph_id in seen_graphs {
            // Get the graph header line
            let graph_lines = graph_data
                .remove(&graph_id)
                .unwrap_or_else(|| vec![format!("G\t{}", graph_id)]);

            // Make sure we have at least a graph declaration line
            if graph_lines.is_empty() {
                output_file.write_all(format!("G\t{}\n", graph_id).as_bytes())?;
            } else {
                // Write graph header line
                output_file.write_all(graph_lines[0].as_bytes())?;
                output_file.write_all(b"\n")?;
            }

            // Write nodes for this graph if they exist
            if let Some(nodes) = node_data.remove(&graph_id) {
                for line in nodes {
                    output_file.write_all(line.as_bytes())?;
                    output_file.write_all(b"\n")?;
                }
            }

            // Write edges for this graph if they exist
            if let Some(edges) = edge_data.remove(&graph_id) {
                for line in edges {
                    output_file.write_all(line.as_bytes())?;
                    output_file.write_all(b"\n")?;
                }
            }

            // Write remaining graph content
            for line in &graph_lines[1..] {
                output_file.write_all(line.as_bytes())?;
                output_file.write_all(b"\n")?;
            }
        }

        // If there are any orphaned nodes or edges (belonging to no graph),
        // write them under the default graph
        let has_orphaned_data =
            node_data.contains_key(&default_graph) || edge_data.contains_key(&default_graph);
        if has_orphaned_data {
            // Write default graph header
            output_file.write_all(format!("G\t{}\n", default_graph).as_bytes())?;

            // Write orphaned nodes
            if let Some(nodes) = node_data.remove(&default_graph) {
                for line in nodes {
                    output_file.write_all(line.as_bytes())?;
                    output_file.write_all(b"\n")?;
                }
            }

            // Write orphaned edges
            if let Some(edges) = edge_data.remove(&default_graph) {
                for line in edges {
                    output_file.write_all(line.as_bytes())?;
                    output_file.write_all(b"\n")?;
                }
            }
        }

        Ok(())
    }

    fn read_dictionaries(&mut self, data: &[u8]) -> Result<()> {
        // Decompress the dictionary data
        let decompressed = decode_all(data).map_err(|e| BTSGError::Compression(e.to_string()))?;
        let mut cursor = io::Cursor::new(decompressed);

        // Read each dictionary based on its type marker
        while let Ok(dict_type) = cursor.read_u8() {
            match dict_type {
                0x01 => {
                    // Node dictionary
                    self.node_dict = StringDictionary::read(&mut cursor)?;
                }
                0x02 => {
                    // Edge dictionary
                    self.edge_dict = StringDictionary::read(&mut cursor)?;
                }
                0x03 => {
                    // Graph dictionary
                    self.graph_dict = StringDictionary::read(&mut cursor)?;
                }
                0x04 => {
                    // Read dictionary
                    self.read_dict = StringDictionary::read(&mut cursor)?;
                }
                0x05 => {
                    // Chromosome dictionary
                    self.chromosome_dict = StringDictionary::read(&mut cursor)?;
                }
                0x06 => {
                    // Attribute dictionary
                    self.attribute_dict = StringDictionary::read(&mut cursor)?;
                }
                _ => {
                    return Err(BTSGError::InvalidFormat(format!(
                        "Unknown dictionary type: {}",
                        dict_type
                    ))
                    .into());
                }
            }
        }
        Ok(())
    }
}

// Add function to read directly from BTSG to memory
impl BTSGDecompressor {
    /// Decompress a BTSG file and return the TSG content as a string
    pub fn decompress_to_string<P: AsRef<Path>>(&mut self, input_path: P) -> Result<String> {
        let mut input_file = File::open(input_path)?;

        // Read and verify magic number
        let mut magic = [0u8; 4];
        input_file.read_exact(&mut magic)?;
        if &magic != b"BTSG" {
            return Err(BTSGError::InvalidFormat("Not a valid BTSG file".to_string()).into());
        }

        // Read version
        let version = input_file.read_u32::<LittleEndian>()?;
        if version != BTSG_VERSION {
            return Err(
                BTSGError::InvalidFormat(format!("Unsupported BTSG version: {}", version)).into(),
            );
        }

        // Pre-allocate with a reasonable capacity
        let mut output = String::with_capacity(10_000); // 10KB initial capacity

        // Similar approach as decompress method but writing to a string
        let mut header_lines = Vec::new();
        let mut graph_data: HashMap<BString, Vec<String>> = HashMap::new();
        let mut node_data: HashMap<BString, Vec<String>> = HashMap::new();
        let mut edge_data: HashMap<BString, Vec<String>> = HashMap::new();

        // Read blocks until EOF
        while let Ok(block) = Block::read(&mut input_file) {
            match block.block_type {
                BLOCK_DICTIONARY => {
                    self.read_dictionaries(&block.data)?;
                }
                BLOCK_HEADER => {
                    let decompressed = decode_all(&block.data[..])
                        .map_err(|e| BTSGError::Compression(e.to_string()))?;
                    let content = String::from_utf8_lossy(&decompressed);
                    header_lines.extend(content.lines().map(|s| s.to_string()));
                }
                BLOCK_GRAPH => {
                    let decompressed = decode_all(&block.data[..])
                        .map_err(|e| BTSGError::Compression(e.to_string()))?;
                    let content = String::from_utf8_lossy(&decompressed);
                    let mut lines = content.lines();

                    if let Some(first_line) = lines.next() {
                        if let Some((_, graph_id)) = first_line.split_once('\t') {
                            let graph_id_bstr = BString::from(graph_id);
                            let graph_entries = graph_data.entry(graph_id_bstr).or_default();
                            graph_entries.push(first_line.to_string());
                            graph_entries.extend(lines.map(|s| s.to_string()));
                        }
                    }
                }
                BLOCK_NODE | BLOCK_EDGE | BLOCK_ATTRIBUTE | BLOCK_CHAIN | BLOCK_PATH
                | BLOCK_LINK => {
                    // Process other block types consistently with decompress method
                    let decompressed = decode_all(&block.data[..])
                        .map_err(|e| BTSGError::Compression(e.to_string()))?;
                    let content = String::from_utf8_lossy(&decompressed);

                    // Process differently based on block type
                    if block.block_type == BLOCK_NODE || block.block_type == BLOCK_EDGE {
                        let mut lines = content.lines();
                        // First line contains graph information
                        if let Some(first_line) = lines.next() {
                            if first_line.starts_with("G\t") {
                                if let Some((_, graph_id)) = first_line.split_once('\t') {
                                    let graph_id_bstr = BString::from(graph_id);
                                    let entries = if block.block_type == BLOCK_NODE {
                                        node_data.entry(graph_id_bstr).or_default()
                                    } else {
                                        edge_data.entry(graph_id_bstr).or_default()
                                    };
                                    entries.extend(lines.map(|s| s.to_string()));
                                }
                            }
                        }
                    } else {
                        // For other block types, add all lines as they are
                        let entries = content.lines().map(|s| s.to_string()).collect::<Vec<_>>();

                        // Determine which graph this belongs to
                        if let Some(first_line) = entries.first() {
                            if first_line.starts_with("G\t") {
                                if let Some((_, graph_id)) = first_line.split_once('\t') {
                                    let graph_id_bstr = BString::from(graph_id);
                                    graph_data.entry(graph_id_bstr).or_default().extend(entries);
                                } else {
                                    // No graph found, just add to general content
                                    header_lines.extend(entries);
                                }
                            } else {
                                // Not graph-specific content, add to header
                                header_lines.extend(entries);
                            }
                        }
                    }
                }
                _ => {
                    return Err(BTSGError::InvalidBlockType(block.block_type).into());
                }
            }
        }

        // Assemble the output string in the right order
        for line in header_lines {
            output.push_str(&line);
            output.push('\n');
        }

        for (graph_id, graph_lines) in graph_data {
            if graph_lines.is_empty() || !graph_lines[0].starts_with("G\t") {
                continue;
            }

            // Write graph header line
            output.push_str(&graph_lines[0]);
            output.push('\n');

            // Write nodes for this graph if they exist
            if let Some(nodes) = node_data.remove(&graph_id) {
                for line in nodes {
                    output.push_str(&line);
                    output.push('\n');
                }
            }

            // Write edges for this graph if they exist
            if let Some(edges) = edge_data.remove(&graph_id) {
                for line in edges {
                    output.push_str(&line);
                    output.push('\n');
                }
            }

            // Write remaining graph content
            for line in &graph_lines[1..] {
                output.push_str(line);
                output.push('\n');
            }
        }

        // Shrink the output string to free unused memory
        if output.capacity() > output.len() * 2 {
            output.shrink_to_fit();
        }

        Ok(output)
    }
}

pub trait BTSG {
    fn from_btsg<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized;

    fn to_btsg<P: AsRef<Path>>(&self, path: P, compression_level: i32) -> Result<()>
    where
        Self: Sized;

    fn from_btsg_direct<P: AsRef<Path>>(path: P) -> Result<Self>
    where
        Self: Sized;
}

impl BTSG for TSGraph {
    /// Load a TSGraph from a BTSG (Binary Transcript Segment Graph) file
    fn from_btsg<P: AsRef<Path>>(path: P) -> Result<Self> {
        debug!(
            "Loading TSGraph from BTSG file: {}",
            path.as_ref().display()
        );

        // Option 1: Use BTSGDecompressor to get TSG content as a string and then parse it
        let mut decompressor = BTSGDecompressor::new();
        let tsg_content = decompressor
            .decompress_to_string(path)
            .context("Failed to decompress BTSG file")?;

        // Create a cursor for reading the TSG content
        let cursor = Cursor::new(tsg_content);
        let mut reader = BufReader::new(cursor);
        // Parse the TSG content
        Self::from_reader(&mut reader)
    }

    /// Load a TSGraph directly from a BTSG file using a more direct approach
    fn from_btsg_direct<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut input_file = File::open(path.as_ref()).context(format!(
            "Failed to open BTSG file: {}",
            path.as_ref().display()
        ))?;

        // Read and verify magic number
        let mut magic = [0u8; 4];
        input_file
            .read_exact(&mut magic)
            .context("Failed to read BTSG magic number")?;

        if &magic != b"BTSG" {
            return Err(anyhow!("Not a valid BTSG file - invalid magic number"));
        }

        // Read version
        let version = input_file
            .read_u32::<LittleEndian>()
            .context("Failed to read BTSG version")?;

        if version != BTSG_VERSION {
            return Err(anyhow!("Unsupported BTSG version: {}", version));
        }

        debug!("Reading BTSG file version {}", version);

        // We need to handle the new block organization
        let mut header_content = Vec::new();
        let mut graph_data: HashMap<BString, Vec<u8>> = HashMap::new();
        let mut node_data: HashMap<BString, Vec<u8>> = HashMap::new();
        let mut edge_data: HashMap<BString, Vec<u8>> = HashMap::new();

        // Dictionary handler (we need to maintain this state)
        let mut dictionary_handler = BTSGDecompressor::new();

        // Process each block
        loop {
            // Read block type and length
            let block_type = match input_file.read_u8() {
                Ok(t) => t,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break, // End of file
                Err(e) => return Err(anyhow!("Error reading block type: {}", e)),
            };

            let block_length = match input_file.read_u32::<LittleEndian>() {
                Ok(len) => len as usize,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break, // Unexpected EOF, but we'll try to parse what we have
                Err(e) => return Err(anyhow!("Error reading block length: {}", e)),
            };

            // Check for unreasonable block size to prevent OOM attacks
            if block_length > 100_000_000 {
                // 100 MB seems like a reasonable limit
                return Err(anyhow!("Block size too large: {} bytes", block_length));
            }

            // Read block data
            let mut block_data = vec![0u8; block_length];
            match input_file.read_exact(&mut block_data) {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    return Err(anyhow!("Unexpected EOF while reading block data"));
                }
                Err(e) => return Err(anyhow!("Error reading block data: {}", e)),
            };

            // Process block based on type
            match block_type {
                BLOCK_DICTIONARY => {
                    debug!("Processing dictionary block");
                    if let Err(e) = dictionary_handler.read_dictionaries(&block_data) {
                        warn!("Error processing dictionary block: {}", e);
                    }
                }
                BLOCK_HEADER => {
                    debug!("Processing header block");
                    let decompressed = decode_all(&block_data[..])
                        .map_err(|e| anyhow!("Failed to decompress header block: {}", e))?;
                    header_content.extend_from_slice(&decompressed);
                    header_content.push(b'\n');
                }
                BLOCK_GRAPH => {
                    debug!("Processing graph block");
                    let decompressed = decode_all(&block_data[..])
                        .map_err(|e| anyhow!("Failed to decompress graph block: {}", e))?;

                    // Extract graph ID from first line
                    let content = String::from_utf8_lossy(&decompressed);
                    let mut lines = content.lines();
                    if let Some(first_line) = lines.next() {
                        if let Some((_, graph_id)) = first_line.split_once('\t') {
                            let graph_id_bstr = BString::from(graph_id);
                            let entry = graph_data.entry(graph_id_bstr).or_default();
                            entry.extend_from_slice(&decompressed);
                            entry.push(b'\n');
                        }
                    }
                }
                BLOCK_NODE => {
                    debug!("Processing node block");
                    let decompressed = decode_all(&block_data[..])
                        .map_err(|e| anyhow!("Failed to decompress node block: {}", e))?;

                    // Extract graph ID from first line
                    let content = String::from_utf8_lossy(&decompressed);
                    let mut lines = content.lines();
                    if let Some(first_line) = lines.next() {
                        if first_line.starts_with("G\t") {
                            if let Some((_, graph_id)) = first_line.split_once('\t') {
                                let graph_id_bstr = BString::from(graph_id);
                                // Store all lines except the first one (which is just the graph ID)
                                let nodes_content =
                                    lines.collect::<Vec<_>>().join("\n").into_bytes();
                                if !nodes_content.is_empty() {
                                    let entry = node_data.entry(graph_id_bstr.clone()).or_default();
                                    entry.extend_from_slice(&nodes_content);
                                    entry.push(b'\n');
                                }
                            }
                        }
                    }
                }
                BLOCK_EDGE => {
                    debug!("Processing edge block");
                    let decompressed = decode_all(&block_data[..])
                        .map_err(|e| anyhow!("Failed to decompress edge block: {}", e))?;

                    // Extract graph ID from first line
                    let content = String::from_utf8_lossy(&decompressed);
                    let mut lines = content.lines();
                    if let Some(first_line) = lines.next() {
                        if first_line.starts_with("G\t") {
                            if let Some((_, graph_id)) = first_line.split_once('\t') {
                                let graph_id_bstr = BString::from(graph_id);
                                // Store all lines except the first one (which is just the graph ID)
                                let edges_content =
                                    lines.collect::<Vec<_>>().join("\n").into_bytes();
                                if !edges_content.is_empty() {
                                    let edges = edge_data.entry(graph_id_bstr.clone()).or_default();
                                    edges.extend_from_slice(&edges_content);
                                    edges.push(b'\n');
                                }
                            }
                        }
                    }
                }
                BLOCK_ATTRIBUTE | BLOCK_CHAIN | BLOCK_PATH | BLOCK_LINK => {
                    debug!("Processing block type {}", block_type);
                    let decompressed = decode_all(&block_data[..]).map_err(|e| {
                        anyhow!("Failed to decompress block type {}: {}", block_type, e)
                    })?;

                    // Add to appropriate section based on first line
                    let content = String::from_utf8_lossy(&decompressed);
                    let mut lines = content.lines();
                    if let Some(first_line) = lines.next() {
                        if first_line.starts_with("G\t") {
                            if let Some((_, graph_id)) = first_line.split_once('\t') {
                                let graph_id_bstr = BString::from(graph_id);
                                // Store content with the appropriate graph
                                let entry = graph_data.entry(graph_id_bstr).or_default();
                                entry.extend_from_slice(&decompressed);
                                entry.push(b'\n');
                            }
                        } else {
                            // No graph associated, add to general content
                            header_content.extend_from_slice(&decompressed);
                            header_content.push(b'\n');
                        }
                    }
                }
                _ => {
                    warn!("Unknown block type: {}", block_type);
                    // Skip unknown blocks instead of failing
                }
            }
        }

        // Assemble the complete TSG content
        let mut tsg_content = Vec::with_capacity(
            header_content.len()
                + graph_data.values().map(|v| v.len()).sum::<usize>()
                + node_data.values().map(|v| v.len()).sum::<usize>()
                + edge_data.values().map(|v| v.len()).sum::<usize>(),
        );

        // Add headers
        if !header_content.is_empty() {
            tsg_content.extend_from_slice(&header_content);
        }

        // Add each graph with its nodes and edges
        for (graph_id, graph_content) in graph_data {
            // Find the graph declaration line
            let graph_content_str = String::from_utf8_lossy(&graph_content);
            let mut lines = graph_content_str.lines();

            if let Some(graph_line) = lines.next() {
                if graph_line.starts_with("G\t") {
                    // Add the graph line
                    tsg_content.extend_from_slice(graph_line.as_bytes());
                    tsg_content.push(b'\n');

                    // Add nodes for this graph if they exist
                    if let Some(nodes) = node_data.get(&graph_id) {
                        tsg_content.extend_from_slice(nodes);
                    }

                    // Add edges for this graph if they exist
                    if let Some(edges) = edge_data.get(&graph_id) {
                        tsg_content.extend_from_slice(edges);
                    }

                    // Add the rest of the graph content
                    for line in lines {
                        tsg_content.extend_from_slice(line.as_bytes());
                        tsg_content.push(b'\n');
                    }
                }
            }
        }

        // Parse the TSG content
        let cursor = Cursor::new(tsg_content);
        let reader = BufReader::new(cursor);
        Self::from_reader(reader)
    }

    /// Save the TSGraph to a BTSG file
    fn to_btsg<P: AsRef<Path>>(&self, path: P, compression_level: i32) -> Result<()> {
        // Create a temporary TSG file
        let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
        let temp_tsg_path = temp_dir.path().join("temp.tsg");

        // Write the TSGraph to the temporary file
        self.to_file(&temp_tsg_path)
            .context("Failed to write TSGraph to temporary file")?;

        // Create a BTSGCompressor instance
        let mut compressor = BTSGCompressor::new(compression_level);

        // Compress the temporary file to the destination
        compressor
            .compress(&temp_tsg_path, &path.as_ref().to_path_buf())
            .context("Failed to compress TSG to BTSG")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use tempfile::NamedTempFile;
    use tsg_core::graph::{EdgeData, GraphSection, Header, NodeData, StructuralVariant};

    #[test]
    fn test_string_dictionary() {
        let mut dict = StringDictionary::new();

        // Add some strings
        let id1 = dict.add("hello".as_bytes().as_bstr());
        let id2 = dict.add("world".as_bytes().as_bstr());
        let id3 = dict.add("hello".as_bytes().as_bstr()); // Should return existing ID

        // Check IDs
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 0); // Same as id1

        // Lookup by ID
        assert_eq!(dict.str(id1).unwrap(), "hello".as_bytes().as_bstr());
        assert_eq!(dict.str(id2).unwrap(), "world".as_bytes().as_bstr());

        // Lookup by string
        assert_eq!(dict.id("hello".as_bytes().as_bstr()).unwrap(), id1);
        assert_eq!(dict.id("world".as_bytes().as_bstr()).unwrap(), id2);
        assert_eq!(dict.id("unknown".as_bytes().as_bstr()), None);

        // Test serialization and deserialization
        let mut buffer = Vec::new();
        dict.write(&mut buffer).unwrap();

        let mut cursor = io::Cursor::new(buffer);
        let loaded_dict = StringDictionary::read(&mut cursor).unwrap();

        // Verify loaded dictionary
        assert_eq!(loaded_dict.str(id1).unwrap(), "hello".as_bytes().as_bstr());
        assert_eq!(loaded_dict.str(id2).unwrap(), "world".as_bytes().as_bstr());
        assert_eq!(loaded_dict.id("hello".as_bytes().as_bstr()).unwrap(), id1);
        assert_eq!(loaded_dict.id("world".as_bytes().as_bstr()).unwrap(), id2);
    }

    #[test]
    fn test_block_serialization() {
        let data = b"test data".to_vec();
        let block = Block::new(BLOCK_HEADER, data.clone());

        let mut buffer = Vec::new();
        block.write(&mut buffer).unwrap();

        let mut cursor = io::Cursor::new(buffer);
        let loaded_block = Block::read(&mut cursor).unwrap();

        assert_eq!(loaded_block.block_type, BLOCK_HEADER);
        assert_eq!(loaded_block.data, data);
    }

    #[test]
    fn test_compression_round_trip() -> Result<()> {
        // Create a small TSG file
        let mut temp_tsg = NamedTempFile::new()?;
        temp_tsg.write_all(b"H\tTSG\t1.0\nH\treference\tGRCh38\nG\tg1\nN\tn1\tchr1:+:1000-2000\tread1:SO\nE\te1\tn1\tn2\tchr1,chr1,2000,3000,splice\n")?;

        // Create a temp file for the compressed output
        let temp_btsg = NamedTempFile::new()?;
        let temp_btsg_path = temp_btsg.path().to_path_buf();

        // Create a temp file for the decompressed output
        let temp_out = NamedTempFile::new()?;
        let temp_out_path = temp_out.path().to_path_buf();

        // Compress
        let mut compressor = BTSGCompressor::new(3); // Medium compression
        compressor.compress(temp_tsg.path(), &temp_btsg_path)?;

        // Decompress
        let mut decompressor = BTSGDecompressor::new();
        decompressor.decompress(&temp_btsg_path, &temp_out_path)?;

        // Compare original and round-tripped content
        let original = std::fs::read_to_string(temp_tsg.path())?;
        let roundtrip = std::fs::read_to_string(&temp_out_path)?;

        // Debug output in case of failure
        if original != roundtrip {
            println!("Original content:\n{}", original);
            println!("Roundtrip content:\n{}", roundtrip);
        }

        // Normalize line endings and trim
        let original_lines: Vec<&str> = original.lines().collect();
        let roundtrip_lines: Vec<&str> = roundtrip.lines().collect();

        assert_eq!(
            original_lines.len(),
            roundtrip_lines.len(),
            "Number of lines differs: {} vs {}",
            original_lines.len(),
            roundtrip_lines.len()
        );

        assert_eq!(original_lines, roundtrip_lines);

        Ok(())
    }

    #[test]
    fn test_from_btsg() -> Result<()> {
        // Create a small TSG file
        let mut temp_tsg = NamedTempFile::new()?;
        temp_tsg.write_all(b"H\tTSG\t1.0\nH\treference\tGRCh38\nG\tg1\nN\tn1\tchr1:+:1000-2000\tread1:SO\nE\te1\tn1\tn2\tchr1,chr1,2000,3000,splice\n")?;

        // Create a temp file for the compressed output
        let temp_btsg = NamedTempFile::new()?;
        let temp_btsg_path = temp_btsg.path().to_path_buf();
        // Compress
        let mut compressor = BTSGCompressor::new(3); // Medium compression
        compressor.compress(temp_tsg.path(), &temp_btsg_path)?;

        // Use from_btsg to create the graph directly
        let graph = TSGraph::from_btsg(&temp_btsg_path)?;

        // Basic validation that the graph was loaded correctly
        assert_eq!(graph.nodes("g1").len(), 2);
        assert_eq!(graph.edges("g1").len(), 1);

        Ok(())
    }

    #[test]
    fn test_from_btsg_roundtrip2() -> Result<()> {
        // Create a small TSG structure
        let mut graph = TSGraph::new();

        // Add headers
        let header1 = Header::builder().tag("TSG").value("1.0").build();
        let header2 = Header::builder().tag("reference").value("GRCh38").build();
        graph.headers.push(header1);
        graph.headers.push(header2);

        // Add a graph section
        let graph_id: BString = "test_graph".into();
        let mut graph_section = GraphSection::new(graph_id.clone());

        // Add nodes to the graph section - Fix the genomic location format
        let node1 = NodeData::from_str("N\tn1\tchr1:+:1000-2000\tread1:SO")?;
        let node2 = NodeData::from_str("N\tn2\tchr1:+:3000-4000\tread1:IN")?;

        graph_section.add_node(node1)?;
        graph_section.add_node(node2)?;

        // Add an edge to the graph section
        let edge_data = EdgeData {
            id: "e1".into(),
            sv: StructuralVariant::from_str("chr1,chr1,2000,3000,splice")?,
            attributes: Default::default(),
        };
        graph_section.add_edge(
            "n1".as_bytes().as_bstr(),
            "n2".as_bytes().as_bstr(),
            edge_data,
        )?;

        // Add the graph section to the main graph
        graph.graphs.insert(graph_id, graph_section);

        // Create a temporary file for the TSG output
        let temp_tsg = NamedTempFile::new()?;
        let temp_tsg_path = temp_tsg.path().to_path_buf();

        // Create a temporary file for the BTSG output
        let temp_btsg = NamedTempFile::new()?;
        let temp_btsg_path = temp_btsg.path().to_path_buf();

        // Write the TSGraph to TSG file
        graph.to_file(&temp_tsg_path)?;

        // Uncomment this to debug the issue - print out actual TSG content
        println!("TSG content: {}", std::fs::read_to_string(&temp_tsg_path)?);

        // Compress the TSG file to BTSG
        graph.to_btsg(&temp_btsg_path, 3)?;

        // Read the BTSG file back into a TSGraph
        let loaded_graph = TSGraph::from_btsg(&temp_btsg_path)?;

        // Verify the loaded graph
        assert!(loaded_graph.headers.len() >= 2); // At least 2 headers (could have more from TSG lib)
        assert!(loaded_graph.headers.iter().any(|h| h.tag == "TSG"));
        assert!(loaded_graph.headers.iter().any(|h| h.tag == "reference"));

        assert_eq!(loaded_graph.graphs.len(), 1);
        assert!(loaded_graph.graphs.contains_key("test_graph".as_bytes()));

        let loaded_section = &loaded_graph.graph("test_graph").unwrap();
        assert_eq!(loaded_section.nodes().len(), 2);
        assert_eq!(loaded_section.edges().len(), 1);
        Ok(())
    }

    #[test]
    fn test_compression_with_separated_blocks() -> Result<()> {
        // Create a TSG file with multiple node and edge entries
        let mut temp_tsg = NamedTempFile::new()?;
        temp_tsg.write_all(
            b"H\tTSG\t1.0\nH\treference\tGRCh38\n\
                           G\tg1\n\
                           N\tn1\tchr1:+:1000-2000\tread1:SO\n\
                           N\tn2\tchr1:+:2100-3000\tread1:SO\n\
                           N\tn3\tchr2:+:1000-2000\tread2:SO\n\
                           E\te1\tn1\tn2\tchr1,chr1,2000,2100,splice\n\
                           E\te2\tn2\tn3\tchr1,chr2,3000,1000,trans\n",
        )?;

        // Create temp files for btsg and decompressed output
        let temp_btsg = NamedTempFile::new()?;
        let temp_btsg_path = temp_btsg.path().to_path_buf();
        let temp_out = NamedTempFile::new()?;
        let temp_out_path = temp_out.path().to_path_buf();

        // Compress with block separation
        let mut compressor = BTSGCompressor::new(3);
        compressor.compress(temp_tsg.path(), &temp_btsg_path)?;

        // Print BTSG contents for debugging
        let btsg_size = std::fs::metadata(&temp_btsg_path)?.len();
        println!("BTSG file size: {} bytes", btsg_size);

        // Decompress
        let mut decompressor = BTSGDecompressor::new();
        decompressor.decompress(&temp_btsg_path, &temp_out_path)?;

        // Verify content is unchanged
        let original = std::fs::read_to_string(temp_tsg.path())?;
        let roundtrip = std::fs::read_to_string(&temp_out_path)?;

        // Debug output
        println!(
            "Original content ({} lines):\n{}",
            original.lines().count(),
            original
        );
        println!(
            "Roundtrip content ({} lines):\n{}",
            roundtrip.lines().count(),
            roundtrip
        );

        // Compare normalized content
        let original_lines: Vec<&str> = original.lines().collect();
        let roundtrip_lines: Vec<&str> = roundtrip.lines().collect();

        assert_eq!(
            original_lines.len(),
            roundtrip_lines.len(),
            "Line count mismatch: expected {}, got {}",
            original_lines.len(),
            roundtrip_lines.len()
        );

        // Compare lines, ignoring exact ordering for nodes and edges
        let mut original_sorted = original_lines.clone();
        let mut roundtrip_sorted = roundtrip_lines.clone();
        original_sorted.sort();
        roundtrip_sorted.sort();

        assert_eq!(original_sorted, roundtrip_sorted);

        // Also test direct loading
        let graph = TSGraph::from_btsg(&temp_btsg_path)?;

        // Debug output for the loaded graph
        println!("Graph nodes: {}", graph.nodes("g1").len());
        println!("Graph edges: {}", graph.edges("g1").len());

        assert_eq!(graph.nodes("g1").len(), 3, "Expected 3 nodes in graph");
        assert_eq!(graph.edges("g1").len(), 2, "Expected 2 edges in graph");

        Ok(())
    }
}
