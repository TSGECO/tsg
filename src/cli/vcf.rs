use std::path::{Path, PathBuf};

use crate::graph::TSGraph;
use crate::io;
use anyhow::Result;
use tracing::info;

pub fn to_vcf<P: AsRef<Path>>(input: P, output: Option<PathBuf>) -> Result<()> {
    let tsg_graph = TSGraph::from_file(input.as_ref())?;
    let output_path = match output {
        Some(path) => path,
        None => {
            let mut output = input.as_ref().to_path_buf();
            output.set_extension("vcf");
            output
        }
    };

    info!("Writing VCF to: {}", output_path.display());
    io::to_vcf(&tsg_graph, output_path)?;
    Ok(())
}
