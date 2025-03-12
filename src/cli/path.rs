use std::io::Write;
use std::path::{Path, PathBuf};

use crate::graph::TSGraph;
use anyhow::Result;
use tracing::info;

// traverse the graph and output the path to the output file
// the output file is plain text file
// each line is a path
// P transcript1	n1+	e1+	n3+	e2+	n4+

pub fn traverse<P: AsRef<Path>>(input: P, write_path: bool, output: Option<PathBuf>) -> Result<()> {
    let tsg_graph = TSGraph::from_file(input.as_ref())?;
    let output_path = match output {
        Some(path) => path,
        None => {
            let mut output = input.as_ref().to_path_buf();
            output.set_extension("path.txt");
            output
        }
    };

    let paths = tsg_graph.traverse()?;
    let file = std::fs::File::create(&output_path)?;
    let mut writer = std::io::BufWriter::new(file);

    info!("Writing path to file: {:?}", output_path);

    for path in paths {
        if write_path {
            // write the path
            writer.write_all(format!("{}\n", path).as_bytes())?;
        } else {
            // only write the path id
            writer.write_all(format!("{}\n", path.id().unwrap()).as_bytes())?;
        }
    }
    Ok(())
}
