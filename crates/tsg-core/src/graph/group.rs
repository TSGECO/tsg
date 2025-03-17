use std::fmt;
use std::{io, str::FromStr};

use ahash::HashMap;
use bstr::BString;

use super::Attribute;

/// Orientation of an element in an ordered group
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Forward,
    Reverse,
}

/// Reference to a graph element with optional orientation
#[derive(Debug, Clone)]
pub struct OrientedElement {
    pub id: BString,
    pub orientation: Option<Orientation>,
}

impl FromStr for OrientedElement {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_suffix('+') {
            Ok(OrientedElement {
                id: stripped.into(),
                orientation: Some(Orientation::Forward),
            })
        } else if let Some(stripped) = s.strip_suffix('-') {
            Ok(OrientedElement {
                id: stripped.into(),
                orientation: Some(Orientation::Reverse),
            })
        } else {
            Ok(OrientedElement {
                id: s.into(),
                orientation: None,
            })
        }
    }
}

impl fmt::Display for OrientedElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.orientation {
            Some(Orientation::Forward) => write!(f, "{}+", self.id),
            Some(Orientation::Reverse) => write!(f, "{}-", self.id),
            None => write!(f, "{}", self.id),
        }
    }
}

/// Group in the transcript segment graph (ordered, unordered, or chain)
#[derive(Debug, Clone)]
pub enum Group {
    Unordered {
        id: BString,
        elements: Vec<BString>,
        attributes: HashMap<BString, Attribute>,
    },
    Ordered {
        id: BString,
        elements: Vec<OrientedElement>,
        attributes: HashMap<BString, Attribute>,
    },
    Chain {
        id: BString,
        elements: Vec<BString>, // Alternating node and edge IDs, starting and ending with nodes
        attributes: HashMap<BString, Attribute>,
    },
}
