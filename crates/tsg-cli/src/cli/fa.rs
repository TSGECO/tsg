use std::path::{Path, PathBuf};

use anyhow::Result;
use std::io::Write;
use tracing::info;
use tsg::graph::TSGraph;

pub fn to_fa<P: AsRef<Path>>(input: P, output: Option<PathBuf>) -> Result<()> {
    let mut tsg_graph = TSGraph::from_file(input.as_ref())?;
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
    tsg::io::to_fa(&mut tsg_graph, &mut writer)?;
    Ok(())
}
