use std::str::FromStr;
use std::{fmt, io};

use ahash::HashMap;
use anyhow::Result;
use bon::Builder;
use bstr::{BString, ByteSlice, ByteVec};
use serde_json::json;

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

impl EdgeData {
    // {
    //     "data": {
    //         "label": "TRA_(False, NovelInsertion(CAACAATGGCCATGAGGGATTCAAGGATTATGC:0))_1",
    //         "weight": 1,
    //         "read_ids": [
    //             "m64135_220621_211550/1114620/ccs"
    //         ],
    //         "breakpoints": "chr2,chr8,85543034,95819124,TRA",
    //         "source": "chr2_85539167_85543034_H_1",
    //         "target": "chr8_95819124_95822730_T_2",
    //         "key": 0
    //     }
    // }
    pub fn to_json(&self, attributes: Option<&[Attribute]>) -> Result<serde_json::Value> {
        let mut data = json!(
            {
                "label": self.id.to_str().unwrap(),
                "read_ids": [],
                "breakpoints": format!("{},{},{},{},{}", self.sv.reference_name1, self.sv.reference_name2, self.sv.breakpoint1, self.sv.breakpoint2, self.sv.sv_type),
                "source": format!("{}_{}_{}_{}", self.sv.reference_name1, self.sv.breakpoint1, self.sv.sv_type, 1),
                "target": format!("{}_{}_{}_{}", self.sv.reference_name2, self.sv.breakpoint2, self.sv.sv_type, 2),
                "key": 0
            }
        );

        for attr in self.attributes.values() {
            data[attr.tag.to_str().unwrap()] = attr.value.to_str().unwrap().into();
        }

        if let Some(attributes) = attributes.as_ref() {
            for attr in attributes.iter() {
                data[attr.tag.to_str().unwrap()] = attr.value.to_str().unwrap().into();
            }
        }

        let json = json!({
            "data": data
        });

        Ok(json)
    }

    pub fn to_vcf(&self, attributes: Option<&[Attribute]>) -> Result<BString> {
        let mut vcf = BString::from("");
        // vcf.push_str(&format!("#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n"));
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
