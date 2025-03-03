mod edge;
mod node;
mod read;
mod write;

pub use edge::*;
pub use node::*;
pub use read::*;
pub use write::*;

use bstr::BString;

/// Represents an optional attribute
#[derive(Debug, Clone)]
pub struct Attribute {
    pub tag: BString,
    pub attribute_type: char,
    pub value: String,
}
