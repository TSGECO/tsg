use anyhow::Result;
use std::{io::Write, path::Path};
use tsg::graph::TSGraph;

pub fn print_header<P: AsRef<Path>>(input: P) -> Result<()> {
    let tsg_graph = TSGraph::from_file(input.as_ref())?;
    let mut writer = std::io::BufWriter::new(std::io::stdout().lock());

    // Print the header
    for header in tsg_graph.headers.iter() {
        writer.write_all(header.to_string().as_bytes())?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;

    Ok(())
}
