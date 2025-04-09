use anyhow::Result;
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use tracing::info;
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

    // write header
    writer
        .write_all(b"gid\tnodes\tedges\tpaths\tmax_path_len\n")
        .unwrap();

    for (id, graph) in tsg_graph.graphs.iter() {
        let node_count = graph.nodes().len();
        let edge_count = graph.edges().len();
        let paths = graph.traverse()?;

        let max_path_len = paths.iter().map(|path| path.nodes.len()).max().unwrap_or(0);
        let path_count = paths.len();

        writer
            .write_all(
                format!(
                    "{}\t{}\t{}\t{}\t{}\n",
                    id, node_count, edge_count, path_count, max_path_len
                )
                .as_bytes(),
            )
            .unwrap();
    }
    Ok(())
}
