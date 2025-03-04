use std::path::{Path, PathBuf};

use crate::graph::TSGraph;
use crate::io;
use anyhow::Result;

pub fn to_fa<P: AsRef<Path>, Q: AsRef<Path>>(
    input: P,
    reference_genome_path: Q,
    output: Option<PathBuf>,
) -> Result<()> {
    let mut tsg_graph = TSGraph::from_file(input.as_ref())?;

    let output_path = match output {
        Some(path) => path,
        None => {
            let mut output = input.as_ref().to_path_buf();
            output.set_extension("fa");
            output
        }
    };
    io::to_fa(&mut tsg_graph, reference_genome_path.as_ref(), &output_path)?;
    Ok(())
}
