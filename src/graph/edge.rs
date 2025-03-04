use std::str::FromStr;
use std::{fmt, io};

use ahash::HashMap;
use anyhow::Result;
use bon::Builder;
use bstr::BString;

use super::Attribute;

#[derive(Debug, Builder, Clone, Default)]
#[builder(on(BString, into))]
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
        // E  <id>  <source_id>  <sink_id>  <SV>
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

impl fmt::Display for StructuralVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{},{},{},{}",
            self.reference_name1,
            self.reference_name2,
            self.breakpoint1,
            self.breakpoint2,
            self.sv_type
        )
    }
}

/// Edge in the transcript segment graph
#[derive(Debug, Clone, Builder, Default)]
#[builder(on(BString, into))]
pub struct EdgeData {
    pub id: BString,
    pub sv: StructuralVariant,
    pub attributes: HashMap<BString, Attribute>,
}
