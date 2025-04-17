use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::info;
use tsg::graph::TSGraph;

// traverse the graph and output the path to the output file
// the output file is plain text file each line is a path
// P transcript1	n1+	e1+	n3+	e2+	n4+
pub fn traverse<P: AsRef<Path>>(input: P, text_path: bool, output: Option<PathBuf>) -> Result<()> {
    let tsg_graph = TSGraph::from_file(input.as_ref())?;
    let mut writer: Box<dyn Write> = match output {
        Some(path) => {
            info!("Writing paths to file: {:?}", path);
            Box::new(std::io::BufWriter::new(std::fs::File::create(path)?))
        }
        None => {
            info!("Writing paths to stdout");
            Box::new(std::io::BufWriter::new(std::io::stdout().lock()))
        }
    };

    let paths = tsg_graph.traverse_all_graphs()?;
    for path in paths {
        if text_path {
            // write the path
            writer.write_all(format!("{}\n", path).as_bytes())?;
        } else {
            // only write the path id
            writer.write_all(format!("{}\n", path.id().unwrap()).as_bytes())?;
        }
    }
    Ok(())
}
