use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::info;
use tsg::graph::TSGraph;

/// Converts a Transcript Segment Graph (TSG) file to GTF format.
///
/// This function reads a TSG file specified by `input`, converts it to GTF format,
/// and writes the result either to the file specified by `output` or to stdout if
/// `output` is `None`.
///
/// # Arguments
///
/// * `input` - A path to the input TSG file
/// * `output` - An optional path to the output GTF file. If `None`, outputs to stdout
///
/// # Returns
///
/// * `Result<()>` - Ok(()) on success, or an error if file operations fail
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
