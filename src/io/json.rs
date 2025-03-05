use std::path::Path;

use crate::graph::TSGraph;
use anyhow::Result;

pub fn to_json<P: AsRef<Path>>(tsg_graph: &TSGraph, output: P) -> Result<()> {
    todo!()
}
