use std::path::{Path, PathBuf};

use anyhow::Result;
use std::io::Write;
use tracing::info;
use tsg::graph::TSGraph;

/// Converts a TSGraph to FA (Finite Automaton) format
///
/// This function reads a TSGraph from the specified input file,
/// converts it to FA format, and writes the result to the specified
/// output or to stdout if no output is provided.
///
/// # Arguments
///
/// * `input` - Path to the input TSGraph file
/// * `output` - Optional path for the output file. If None, output is written to stdout
///
/// # Returns
///
/// * `Result<()>` - Ok if the conversion was successful, Err otherwise
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
