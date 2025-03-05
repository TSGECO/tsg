use std::fmt;

use bon::Builder;
use bstr::BString;

/// Header information in the TSG file
#[derive(Debug, Clone, PartialEq, Builder)]
#[builder(on(BString, into))]
pub struct Header {
    pub tag: BString,
    pub value: BString,
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "H\t{}\t{}", self.tag, self.value)
    }
}
