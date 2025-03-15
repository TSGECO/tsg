use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::info;
use tsg::graph::TSGraph;

pub fn to_gtf<P: AsRef<Path>>(input: P, output: Option<PathBuf>) -> Result<()> {
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
    tsg::io::to_gtf(&tsg_graph, &mut writer)?;
    Ok(())
}
