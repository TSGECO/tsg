use std::io;
use std::str::FromStr;

use ahash::HashMap;
use anyhow::Result;
use bstr::BString;
use derive_builder::Builder;

use super::Attribute;

#[derive(Debug, Builder, Clone)]
pub struct StructuralVariant {
    pub reference_name1: BString,
    pub reference_name2: BString,
    pub breakpoint1: usize,
    pub breakpoint2: usize,
    pub sv_type: BString,
}

impl FromStr for StructuralVariant {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 5 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid SV format: {}", s),
            ));
        }

        let breakpoint1 = parts[2].parse::<usize>().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid breakpoint1: {}", e),
            )
        })?;

        let breakpoint2 = parts[3].parse::<usize>().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid breakpoint2: {}", e),
            )
        })?;

        Ok(StructuralVariant {
            reference_name1: parts[0].into(),
            reference_name2: parts[1].into(),
            breakpoint1,
            breakpoint2,
            sv_type: parts[4].into(),
        })
    }
}

/// Edge in the transcript segment graph
#[derive(Debug, Clone)]
pub struct Edge {
    pub id: BString,
    pub source_id: BString,
    pub sink_id: BString,
    pub sv: StructuralVariant,
    pub attributes: HashMap<String, Attribute>,
}
