use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define input and output file paths

    let src = env::args().nth(1).expect("missing input file");
    let tsg_file = Path::new(&src);
    let btsg_file = Path::new("example.btsg");
    let output_tsg = Path::new("roundtrip.tsg");

    // Compress TSG to BTSG
    println!(
        "Compressing {} to {}...",
        tsg_file.display(),
        btsg_file.display()
    );
    let mut compressor = tsg::btsg::BTSGCompressor::new(3); // Use compression level 3
    compressor.compress(tsg_file, btsg_file)?;

    // Get file sizes for comparison
    let original_size = std::fs::metadata(tsg_file)?.len();
    let compressed_size = std::fs::metadata(btsg_file)?.len();
    let compression_ratio = (original_size as f64) / (compressed_size as f64);

    println!("Original size: {} bytes", original_size);
    println!("Compressed size: {} bytes", compressed_size);
    println!("Compression ratio: {:.2}x", compression_ratio);

    // Decompress BTSG back to TSG
    println!(
        "\nDecompressing {} to {}...",
        btsg_file.display(),
        output_tsg.display()
    );
    let mut decompressor = tsg::btsg::BTSGDecompressor::new();
    decompressor.decompress(btsg_file, output_tsg)?;

    println!("Decompression completed successfully");

    // Using BTSG with TSGraph
    // println!("\nLoading graph directly from BTSG file...");
    // let graph = tsg::graph::TSGraph::from_btsg(btsg_file)?;

    // Display graph information
    // println!(
    //     "Loaded {} graphs with {} nodes and {} edges",
    //     graph.get_graphs().len(),
    //     graph.get_nodes().len(),
    //     graph.get_edges().len()
    // );

    Ok(())
}
