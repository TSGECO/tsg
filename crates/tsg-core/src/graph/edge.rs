use std::str::FromStr;
use std::{fmt, io};

use ahash::HashMap;
use anyhow::Result;
use bon::Builder;
use bstr::{BString, ByteVec};

use super::Attribute;

/// Represents a structural variant in a genomic sequence.
///
/// A structural variant describes a genomic rearrangement between two locations,
/// potentially on different reference sequences.
///
/// # Fields
///
/// * `reference_name1` - The name of the first reference sequence.
/// * `reference_name2` - The name of the second reference sequence.
/// * `breakpoint1` - The position on the first reference sequence where the variant occurs.
/// * `breakpoint2` - The position on the second reference sequence where the variant occurs.
/// * `sv_type` - The type of structural variant (e.g., "DEL", "INV", "DUP", "TRA").
///
/// # Examples
///
/// ```
/// use bstr::BString;
/// use tsg_core::graph::StructuralVariant;
///
/// let sv = StructuralVariant {
///     reference_name1: BString::from("chr1"),
///     reference_name2: BString::from("chr1"),
///     breakpoint1: 1000,
///     breakpoint2: 5000,
///     sv_type: BString::from("DEL"),
/// };
///
/// let sv_from_builder = StructuralVariant::builder()
///    .reference_name1("chr1")
///    .reference_name2("chr1")
///    .breakpoint1(1000)
///    .breakpoint2(5000)
///    .sv_type(BString::from("DEL"))
///    .build();
/// ```
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

/// Represents an edge in a transcript segment graph.
///
/// Each edge contains a structural variant and additional attributes.
///
/// # Fields
///
/// * `id` - The unique identifier for this edge.
/// * `sv` - The structural variant associated with this edge.
/// * `attributes` - A collection of additional attributes for this edge.
///
#[derive(Debug, Clone, Builder, Default)]
#[builder(on(BString, into))]
pub struct EdgeData {
    pub id: BString,
    pub sv: StructuralVariant,
    pub attributes: HashMap<BString, Attribute>,
}

impl EdgeData {
    pub fn to_vcf(&self, attributes: Option<&[Attribute]>) -> Result<BString> {
        let mut vcf = BString::from("");
        vcf.push_str(format!(
            "{}\t{}\t{}\t.\t<{}>\t.\t.\tCHR2={};SVEND={};",
            self.sv.reference_name1,
            self.sv.breakpoint1,
            self.id,
            self.sv.sv_type,
            self.sv.reference_name2,
            self.sv.breakpoint2
        ));

        let mut info = BString::from("");
        for attr in self.attributes.values() {
            info.push_str(format!("{}={};", attr.tag, attr.value));
        }

        if let Some(attributes) = attributes {
            for attr in attributes.iter() {
                info.push_str(format!("{}={};", attr.tag, attr.value));
            }
        }

        vcf.push_str(&info);
        Ok(vcf)
    }
}
