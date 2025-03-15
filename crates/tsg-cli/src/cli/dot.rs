use std::{io::Write, path::Path};

use anyhow::Result;
use tracing::info;
use tsg::graph::TSGraph;

pub fn to_dot<P: AsRef<Path>>(input: P, output: Option<P>) -> Result<()> {
    let tsg_graph = TSGraph::from_file(input.as_ref())?;

    info!(
        "parsing {} TSG graph from file: {:?}",
        tsg_graph.graphs.len(),
        input.as_ref()
    );
    let output_path = match output {
        Some(path) => path.as_ref().to_path_buf(),
        None => {
            let input_path = input.as_ref().to_path_buf();
            let parent = input_path.parent().unwrap_or(Path::new("."));
            let stem = input_path
                .file_stem()
                .unwrap_or_else(|| std::ffi::OsStr::new("output"));
            let dot_dir = format!("{}_dot", stem.to_string_lossy());
            parent.join(dot_dir)
        }
    };

    // create a folder for the output if it doesn't exist
    if !output_path.exists() {
        std::fs::create_dir_all(&output_path)?;
    }
    for (id, graph) in tsg_graph.graphs.iter() {
        // create a dot file for each graph under the output directory
        let graph_output_file = output_path.join(format!("{}.dot", id));
        let output_file = std::fs::File::create(graph_output_file)?;
        let mut writer = std::io::BufWriter::new(output_file);
        let dot = graph.to_dot(true, true)?;
        writer.write_all(dot.as_bytes())?;
    }
    Ok(())
}
