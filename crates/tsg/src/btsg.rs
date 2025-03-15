use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};

use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::Path;

use bstr::{BStr, BString, ByteSlice};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use thiserror::Error;
use zstd::{decode_all, encode_all};

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

pub type Result<T> = std::result::Result<T, BTSGError>;

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
        Self {
            str_to_id: HashMap::new(),
            id_to_str: HashMap::new(),
            next_id: 0,
        }
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

    fn get_str(&self, id: u32) -> Option<&BStr> {
        self.id_to_str.get(&id).map(|s| s.as_bstr())
    }

    fn get_id(&self, s: &BStr) -> Option<u32> {
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
            node_dict: StringDictionary::new(),
            edge_dict: StringDictionary::new(),
            graph_dict: StringDictionary::new(),
            read_dict: StringDictionary::new(),
            chromosome_dict: StringDictionary::new(),
            attribute_dict: StringDictionary::new(),
            compression_level,
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

        // Organize data by block type
        let mut header_data = Vec::new();
        let mut graphs: HashMap<BString, Vec<String>> = HashMap::new();
        let mut current_graph: Option<BString> = None;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();
            if fields.is_empty() {
                continue;
            }

            match fields[0] {
                "H" => {
                    // Add to header block
                    header_data.push(line);
                }
                "G" => {
                    // New graph
                    if fields.len() >= 2 {
                        let graph_id = BString::from(fields[1]);
                        current_graph = Some(graph_id.clone());
                        graphs.entry(graph_id).or_default().push(line);
                    }
                }
                "N" | "E" | "A" | "C" | "P" | "L" => {
                    // Add to current graph's data
                    if let Some(ref graph_id) = current_graph {
                        graphs.entry(graph_id.clone()).or_default().push(line);
                    } else {
                        // No current graph, create a default one
                        let default_graph = BString::from("default");
                        current_graph = Some(default_graph.clone());
                        graphs.entry(default_graph).or_default().push(line);
                    }
                }
                _ => {
                    // Unknown record type, skip
                    eprintln!("Warning: Unknown record type: {}", fields[0]);
                }
            }
        }

        // Write header block
        if !header_data.is_empty() {
            let header_block =
                self.create_compressed_block(BLOCK_HEADER, header_data.join("\n"))?;
            header_block.write(&mut output_file)?;
        }

        // Write graph blocks
        for (graph_id, graph_data) in graphs {
            // Create a compressed block for this graph's data
            let graph_block = self.create_compressed_block(
                BLOCK_GRAPH,
                format!("G\t{}\n{}", graph_id, graph_data.join("\n")),
            )?;
            graph_block.write(&mut output_file)?;
        }

        Ok(())
    }

    fn build_dictionaries<P: AsRef<Path>>(&mut self, input_path: P) -> Result<()> {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);

        let mut read_ids = HashSet::new();
        let mut chromosomes = HashSet::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let fields: Vec<&str> = line.split('\t').collect();
            if fields.is_empty() {
                continue;
            }

            match fields[0] {
                "G" => {
                    // Add graph ID to dictionary
                    if fields.len() >= 2 {
                        self.graph_dict.add(fields[1].as_bytes().as_bstr());
                    }
                }
                "N" => {
                    // Add node ID and parse genomic location
                    if fields.len() >= 4 {
                        self.node_dict.add(fields[1].as_bytes().as_bstr());

                        // Extract chromosome from genomic location
                        let genomic_loc = fields[2];
                        if let Some(chr_end) = genomic_loc.find(':') {
                            let chromosome = &genomic_loc[0..chr_end];
                            chromosomes.insert(chromosome.to_string());
                        }

                        // Extract read IDs
                        let reads = fields[3];
                        for read_entry in reads.split(',') {
                            if let Some(colon_pos) = read_entry.find(':') {
                                let read_id = &read_entry[0..colon_pos];
                                read_ids.insert(read_id.to_string());
                            }
                        }
                    }
                }
                "E" => {
                    // Add edge ID and node IDs
                    if fields.len() >= 4 {
                        self.edge_dict.add(fields[1].as_bytes().as_bstr());
                        self.node_dict.add(fields[2].as_bytes().as_bstr());
                        self.node_dict.add(fields[3].as_bytes().as_bstr());
                    }
                }
                "A" => {
                    // Add attribute tag
                    if fields.len() >= 4 {
                        self.attribute_dict.add(fields[3].as_bytes().as_bstr());
                    }
                }
                _ => {}
            }
        }

        // Add all read IDs and chromosomes to dictionaries
        for read_id in read_ids {
            self.read_dict.add(read_id.as_bytes().as_bstr());
        }

        for chromosome in chromosomes {
            self.chromosome_dict.add(chromosome.as_bytes().as_bstr());
        }

        Ok(())
    }

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

        // Create a compressed block
        let compressed = encode_all(&buffer[..], self.compression_level)
            .map_err(|e| BTSGError::Compression(e.to_string()))?;

        Ok(Block::new(BLOCK_DICTIONARY, compressed))
    }

    fn create_compressed_block(&self, block_type: u8, data: String) -> Result<Block> {
        // For graph blocks, we need to ensure proper formatting
        if block_type == BLOCK_GRAPH {
            // The data already contains the G line at the beginning, but we need to make sure
            // it doesn't include it in the subsequent lines as well
            let mut lines = data.lines();

            // Extract the graph declaration line
            if let Some(graph_line) = lines.next() {
                // Rebuild the data without duplicating the graph line
                let mut cleaned_data = String::from(graph_line);

                // Add the rest of the lines, filtering out any additional G lines
                for line in lines {
                    if !line.starts_with("G\t") {
                        cleaned_data.push('\n');
                        cleaned_data.push_str(line);
                    }
                }

                // Compress the cleaned data
                let compressed = encode_all(cleaned_data.as_bytes(), self.compression_level)
                    .map_err(|e| BTSGError::Compression(e.to_string()))?;

                return Ok(Block::new(block_type, compressed));
            }
        }

        // For other block types, proceed as before
        let compressed = encode_all(data.as_bytes(), self.compression_level)
            .map_err(|e| BTSGError::Compression(e.to_string()))?;

        Ok(Block::new(block_type, compressed))
    }
}

/// TSG decompressor - converts BTSG back to TSG format
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
        Self {
            node_dict: StringDictionary::new(),
            edge_dict: StringDictionary::new(),
            graph_dict: StringDictionary::new(),
            read_dict: StringDictionary::new(),
            chromosome_dict: StringDictionary::new(),
            attribute_dict: StringDictionary::new(),
        }
    }

    pub fn decompress<P: AsRef<Path>>(&mut self, input_path: P, output_path: P) -> Result<()> {
        let mut input_file = File::open(input_path)?;

        // Read and verify magic number
        let mut magic = [0u8; 4];
        input_file.read_exact(&mut magic)?;
        if &magic != b"BTSG" {
            return Err(BTSGError::InvalidFormat(
                "Not a valid BTSG file".to_string(),
            ));
        }

        // Read version
        let version = input_file.read_u32::<LittleEndian>()?;
        if version != BTSG_VERSION {
            return Err(BTSGError::InvalidFormat(format!(
                "Unsupported BTSG version: {}",
                version
            )));
        }

        let mut output_file = File::create(output_path)?;

        // Read blocks until EOF
        while let Ok(block) = Block::read(&mut input_file) {
            match block.block_type {
                BLOCK_DICTIONARY => {
                    // Read dictionaries
                    self.read_dictionaries(&block.data)?;
                }
                BLOCK_HEADER => {
                    // Write header data to output
                    let decompressed = decode_all(&block.data[..])
                        .map_err(|e| BTSGError::Compression(e.to_string()))?;
                    output_file.write_all(&decompressed)?;
                    output_file.write_all(b"\n")?;
                }
                BLOCK_GRAPH => {
                    // Write graph data to output, but need to parse properly
                    let decompressed = decode_all(&block.data[..])
                        .map_err(|e| BTSGError::Compression(e.to_string()))?;

                    // Convert to string and parse line by line
                    let content = String::from_utf8_lossy(&decompressed);
                    let mut lines = content.lines();

                    // The first line should be the graph declaration line (G)
                    if let Some(first_line) = lines.next() {
                        // Write the graph declaration line
                        output_file.write_all(first_line.as_bytes())?;
                        output_file.write_all(b"\n")?;

                        // Write the rest of the lines (which don't include the graph line again)
                        for line in lines {
                            output_file.write_all(line.as_bytes())?;
                            output_file.write_all(b"\n")?;
                        }
                    }
                }
                _ => {
                    return Err(BTSGError::InvalidBlockType(block.block_type));
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
                    )));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

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
        assert_eq!(dict.get_str(id1).unwrap(), "hello".as_bytes().as_bstr());
        assert_eq!(dict.get_str(id2).unwrap(), "world".as_bytes().as_bstr());

        // Lookup by string
        assert_eq!(dict.get_id("hello".as_bytes().as_bstr()).unwrap(), id1);
        assert_eq!(dict.get_id("world".as_bytes().as_bstr()).unwrap(), id2);
        assert_eq!(dict.get_id("unknown".as_bytes().as_bstr()), None);

        // Test serialization and deserialization
        let mut buffer = Vec::new();
        dict.write(&mut buffer).unwrap();

        let mut cursor = io::Cursor::new(buffer);
        let loaded_dict = StringDictionary::read(&mut cursor).unwrap();

        // Verify loaded dictionary
        assert_eq!(
            loaded_dict.get_str(id1).unwrap(),
            "hello".as_bytes().as_bstr()
        );
        assert_eq!(
            loaded_dict.get_str(id2).unwrap(),
            "world".as_bytes().as_bstr()
        );
        assert_eq!(
            loaded_dict.get_id("hello".as_bytes().as_bstr()).unwrap(),
            id1
        );
        assert_eq!(
            loaded_dict.get_id("world".as_bytes().as_bstr()).unwrap(),
            id2
        );
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

        // Normalize line endings and trim
        let original_lines: Vec<&str> = original.lines().collect();
        let roundtrip_lines: Vec<&str> = roundtrip.lines().collect();

        assert_eq!(original_lines, roundtrip_lines);

        Ok(())
    }
}
