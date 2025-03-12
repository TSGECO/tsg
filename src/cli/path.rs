use std::path::{Path, PathBuf};

use crate::graph::TSGraph;
use crate::io;
use anyhow::Result;
use tracing::info;

// traverse the graph and output the path to the output file
// the output file is plain text file
// each line is a path
// P	transcript1	n1+	e1+	n3+	e2+	n4+

pub fn path<P: AsRef<Path>>(input: P, output: Option<PathBuf>) -> Result<()> {
    let tsg_graph = TSGraph::from_file(input.as_ref())?;
    let output_path = match output {
        Some(path) => path,
        None => {
            let mut output = input.as_ref().to_path_buf();
            output.set_extension("json");
            output
        }
    };
    Ok(())
}
