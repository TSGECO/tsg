use std::path::{Path, PathBuf};

use crate::graph::TSGraph;
use crate::io;
use anyhow::Result;
use std::io::Write;
use tracing::info;

pub fn to_vcf<P: AsRef<Path>>(input: P, output: Option<PathBuf>) -> Result<()> {
    let tsg_graph = TSGraph::from_file(input.as_ref())?;
    let mut writer: Box<dyn Write> = match output {
        Some(path) => {
            info!("Writing to file: {:?}", path);
            Box::new(std::io::BufWriter::new(std::fs::File::create(path)?))
        }
        None => {
            info!("Writing to stdout");
            Box::new(std::io::BufWriter::new(std::io::stdout().lock()))
        }
    };
    io::to_vcf(&tsg_graph, &mut writer)?;
    Ok(())
}
