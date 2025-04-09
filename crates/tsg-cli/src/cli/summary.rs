use anyhow::Result;
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use tracing::info;
use tsg::graph::GraphAnalysis;
use tsg::graph::TSGraph;

pub fn summary<P: AsRef<Path>>(input: P, output: Option<PathBuf>) -> Result<()> {
    let tsg_graph = TSGraph::from_file(input.as_ref())?;
    info!(
        "parsing {} TSG graph from file: {:?}",
        tsg_graph.graphs.len(),
        input.as_ref()
    );

    let mut writer: Box<dyn Write> = match output {
        Some(path) => {
            info!("Writing summary to file: {:?}", path);
            Box::new(std::io::BufWriter::new(std::fs::File::create(path)?))
        }
        None => {
            info!("Writing summary to stdout");
            Box::new(std::io::BufWriter::new(std::io::stdout().lock()))
        }
    };

    let summary_str = tsg_graph.summarize()?;
    writer.write_all(&summary_str)?;
    writer.flush()?;
    Ok(())
}
