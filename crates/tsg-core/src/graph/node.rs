use std::fmt;
use std::str::FromStr;

use crate::graph::Attribute;
use ahash::HashMap;
use anyhow::Context;
use anyhow::Result;
use bon::Builder;
use bon::builder;
use bstr::BString;
use bstr::ByteSlice;
use rayon::prelude::*;
use serde_json::json;
use std::io;
use tracing::debug;

// Define the interval struct
// []
#[derive(Debug, Builder, Clone)]
pub struct Interval {
    pub start: usize,
    pub end: usize,
}

impl FromStr for Interval {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid exon coordinates format: {}", s),
            ));
        }

        let start = parts[0].parse::<usize>().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid start coordinate: {}", e),
            )
        })?;

        let end = parts[1].parse::<usize>().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid end coordinate: {}", e),
            )
        })?;

        Ok(Self { start, end })
    }
}

#[derive(Debug, Builder, Clone, Default)]
pub struct Exons {
    pub exons: Vec<Interval>,
}

impl FromStr for Exons {
    type Err = io::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let exons = s
            .split(',')
            .map(|x| x.parse())
            .collect::<Result<Vec<Interval>, Self::Err>>()?;
        Ok(Exons { exons })
    }
}

impl fmt::Display for Exons {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let exons = self
            .exons
            .iter()
            .map(|x| format!("{}-{}", x.start, x.end))
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "{}", exons)
    }
}
impl Exons {
    pub fn introns(&self) -> Vec<Interval> {
        let mut introns = Vec::new();
        for i in 0..self.exons.len() - 1 {
            introns.push(Interval {
                start: self.exons[i].end + 1,
                end: self.exons[i + 1].start - 1,
            });
        }
        introns
    }
    pub fn is_empty(&self) -> bool {
        self.exons.is_empty()
    }

    pub fn len(&self) -> usize {
        self.exons.len()
    }

    pub fn span(&self) -> usize {
        self.exons.iter().map(|x| x.end - x.start + 1).sum()
    }

    pub fn first_exon(&self) -> &Interval {
        &self.exons[0]
    }

    pub fn last_exon(&self) -> &Interval {
        &self.exons[self.exons.len() - 1]
    }
}

#[derive(Debug, Clone, Builder, PartialEq)]
#[builder(on(BString, into))]
#[builder(on(ReadIdentity, into))]
pub struct ReadData {
    pub id: BString,
    pub identity: ReadIdentity,
}

impl fmt::Display for ReadData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{:?}", self.id, self.identity)
    }
}

impl FromStr for ReadData {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // <id>:<identity>
        let fields: Vec<&str> = s.split(':').collect();
        if fields.len() != 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid read line format: {}", s),
            ));
        }

        let id: BString = fields[0].into();
        let identity = fields[1].parse()?;
        Ok(Self { id, identity })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReadIdentity {
    SO, // source
    IN, // intermediate
    SI, // sink
}

impl fmt::Display for ReadIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadIdentity::SO => write!(f, "SO"),
            ReadIdentity::IN => write!(f, "IN"),
            ReadIdentity::SI => write!(f, "SI"),
        }
    }
}

impl FromStr for ReadIdentity {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SO" => Ok(ReadIdentity::SO),
            "IN" => Ok(ReadIdentity::IN),
            "SI" => Ok(ReadIdentity::SI),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid read identity: {}", s),
            )),
        }
    }
}

impl From<&str> for ReadIdentity {
    fn from(s: &str) -> Self {
        s.parse().unwrap()
    }
}

/// Represents DNA strand orientation
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Strand {
    #[default]
    Forward,
    Reverse,
}

impl FromStr for Strand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Strand::Forward),
            "-" => Ok(Strand::Reverse),
            _ => Err(anyhow::anyhow!("Invalid strand: {}", s)),
        }
    }
}

impl fmt::Display for Strand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Strand::Forward => write!(f, "+"),
            Strand::Reverse => write!(f, "-"),
        }
    }
}

/// Node in the transcript segment graph
#[derive(Debug, Clone, Default, Builder)]
#[builder(on(BString, into))]
pub struct NodeData {
    pub id: BString,
    pub reference_id: BString,
    pub strand: Strand,
    pub exons: Exons,
    pub reads: Vec<ReadData>,
    pub sequence: Option<BString>,
    pub attributes: HashMap<BString, Attribute>,
}

impl NodeData {
    pub fn reference_start(&self) -> usize {
        self.exons.first_exon().start
    }
    pub fn reference_end(&self) -> usize {
        self.exons.last_exon().end
    }
    /// Converts the node data to a JSON representation
    ///
    /// # Arguments
    /// * `attributes` - Optional additional attributes to include in the JSON
    ///
    /// # Returns
    /// A JSON value representing the node
    pub fn to_json(&self, attributes: Option<&[Attribute]>) -> Result<serde_json::Value> {
        let mut data = json!({
            "chrom": self.reference_id.to_str().unwrap(),
            "ref_start": self.reference_start(),
            "ref_end": self.reference_end(),
            "strand": self.strand.to_string(),
            "exons": format!("[{}]",  self.exons.to_string()),
            "reads": self.reads.par_iter().map(|r| format!("{}", r) ).collect::<Vec<_>>(),
            "id": self.id.to_str().unwrap(),
        });

        for attr in self.attributes.values() {
            data[attr.tag.to_str().unwrap()] = match attr.attribute_type {
                'f' => attr.as_float()?.into(),
                'i' => attr.as_int()?.into(),
                _ => attr.value.to_str().unwrap().into(),
            };
        }

        if let Some(attributes) = attributes.as_ref() {
            for attr in attributes.iter() {
                data[attr.tag.to_str().unwrap()] = match attr.attribute_type {
                    'f' => attr.as_float()?.into(),
                    'i' => attr.as_int()?.into(),
                    _ => attr.value.to_str().unwrap().into(),
                };
            }
        }
        let json = json!({"data": data});
        Ok(json)
    }

    pub fn to_gtf(&self, attributes: Option<&[Attribute]>) -> Result<BString> {
        // chr1    scannls exon    173867960       173867991       .       -       .       exon_id "001"; segment_id "0001"; ptc "1"; ptf "1.0"; transcript_id "3x1"; gene_id "3";
        let mut res = vec![];
        for (idx, exon) in self.exons.exons.iter().enumerate() {
            let mut gtf = String::from("");
            gtf.push_str(self.reference_id.to_str().unwrap());
            gtf.push_str("\ttsg\texon\t");
            gtf.push_str(&format!("{}\t{}\t", exon.start, exon.end));
            gtf.push_str(".\t");
            gtf.push_str(self.strand.to_string().as_str());
            gtf.push_str("\t.\t");
            gtf.push_str(format!("exon_id \"{:03}\"; ", idx + 1).as_str());

            for attr in self.attributes.values() {
                gtf.push_str(format!("{} \"{}\"; ", attr.tag, attr.value).as_str());
            }

            if let Some(attributes) = attributes.as_ref() {
                for attr in attributes.iter().rev() {
                    gtf.push_str(format!("{} \"{}\"; ", attr.tag, attr.value).as_str());
                }
            }
            res.push(gtf);
        }
        Ok(res.join("\n").into())
    }
}

impl fmt::Display for NodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "N\t{}\t{}:{}\t{}\t{}",
            self.id,
            self.reference_id,
            self.exons,
            self.reads
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join(","),
            self.sequence.as_ref().unwrap_or(&"".into())
        )
    }
}

impl FromStr for NodeData {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // N  <rid>:<id>  <chrom>:<strand>:<exons>  <reads>  [<seq>]
        let fields: Vec<&str> = s.split_whitespace().collect();
        if fields.len() < 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid node line format: {}", s),
            ));
        }

        debug!("Parsing node: {}", s);
        let id: BString = fields[1].into();

        let reference_and_exons: Vec<&str> = fields[2].split(":").collect();
        let reference_id = reference_and_exons[0].into();
        let strand = reference_and_exons[1].parse().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse strand: {}", e),
            )
        })?;
        let exons = reference_and_exons[2].parse().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse exons: {}", e),
            )
        })?;

        let reads = fields[3]
            .split(',')
            .map(|s| s.parse().context("failed to parse reads").unwrap())
            .collect::<Vec<_>>();

        let sequence = if fields.len() > 4 && !fields[4].is_empty() {
            Some(fields[4].into())
        } else {
            None
        };

        Ok(NodeData {
            id,
            reference_id,
            strand,
            exons,
            reads,
            sequence,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::HashMapExt;

    #[test]
    fn test_node_from_str() {
        let node1 = NodeData::from_str("N\tn1\tchr1:+:1000-2000\tread1:SO").unwrap();
        assert_eq!(node1.id, "n1");
    }

    #[test]
    fn test_exons_introns() {
        let exons = Exons::from_str("100-200,300-400,500-600").unwrap();
        let introns = exons.introns();
        assert_eq!(introns.len(), 2);
        assert_eq!(introns[0].start, 201);
        assert_eq!(introns[0].end, 299);
        assert_eq!(introns[1].start, 401);
        assert_eq!(introns[1].end, 499);
    }

    #[test]
    fn test_exons_len() {
        let exons = Exons::from_str("100-200,300-400,500-600").unwrap();
        assert_eq!(exons.len(), 3);
    }

    #[test]
    fn test_exons_span() {
        let exons = Exons::from_str("100-200,300-400,500-600").unwrap();
        // (200-100+1) + (400-300+1) + (600-500+1) = 101 + 101 + 101 = 303
        assert_eq!(exons.span(), 303);
    }

    #[test]
    fn test_exons_first_last() {
        let exons = Exons::from_str("100-200,300-400,500-600").unwrap();
        assert_eq!(exons.first_exon().start, 100);
        assert_eq!(exons.first_exon().end, 200);
        assert_eq!(exons.last_exon().start, 500);
        assert_eq!(exons.last_exon().end, 600);
    }

    #[test]
    fn test_node_reference_start_end() {
        let node = NodeData {
            id: "node1".into(),
            reference_id: "chr1".into(),
            exons: Exons {
                exons: vec![
                    Interval {
                        start: 100,
                        end: 200,
                    },
                    Interval {
                        start: 300,
                        end: 400,
                    },
                ],
            },
            ..Default::default()
        };

        assert_eq!(node.reference_start(), 100);
        assert_eq!(node.reference_end(), 400);
    }

    #[test]
    fn test_node_to_json() -> Result<()> {
        let node = NodeData {
            id: "node1".into(),
            reference_id: "chr1".into(),
            strand: Strand::Forward,
            exons: Exons {
                exons: vec![
                    Interval {
                        start: 100,
                        end: 200,
                    },
                    Interval {
                        start: 300,
                        end: 400,
                    },
                ],
            },
            reads: vec![
                ReadData::builder().id("read1").identity("SO").build(),
                ReadData::builder().id("read2").identity("IN").build(),
            ],
            attributes: {
                let mut map = HashMap::new();
                map.insert(
                    "ptc".into(),
                    Attribute {
                        tag: "ptc".into(),
                        attribute_type: 'Z',
                        value: "1".into(),
                    },
                );
                map.insert(
                    "ptf".into(),
                    Attribute {
                        tag: "ptf".into(),
                        attribute_type: 'Z',
                        value: "0.0".into(),
                    },
                );
                map
            },
            ..Default::default()
        };

        let json = node.to_json(None)?;
        println!("{}", json);

        // Check basic structure
        assert!(json.get("data").is_some());
        let data = json["data"].as_object().unwrap();

        // Check fields
        assert_eq!(data["chrom"], "chr1");
        assert_eq!(data["ref_start"], 100);
        assert_eq!(data["ref_end"], 400);
        assert_eq!(data["strand"], "+");
        assert_eq!(data["id"], "node1");
        assert_eq!(data["ptc"], "1");
        assert_eq!(data["ptf"], "0.0");

        // Test with additional attributes
        let additional_attrs = vec![Attribute {
            tag: "is_head".into(),
            attribute_type: 'Z',
            value: "true".into(),
        }];

        let json_with_attrs = node.to_json(Some(&additional_attrs))?;
        let data = json_with_attrs["data"].as_object().unwrap();
        assert_eq!(data["is_head"], "true");

        println!("{}", json_with_attrs);

        Ok(())
    }

    #[test]
    fn test_node_to_gtf() -> Result<()> {
        let node = NodeData {
            id: "node1".into(),
            reference_id: "chr1".into(),
            strand: Strand::Forward,
            exons: Exons {
                exons: vec![
                    Interval {
                        start: 100,
                        end: 200,
                    },
                    Interval {
                        start: 300,
                        end: 400,
                    },
                ],
            },
            attributes: {
                let mut map = HashMap::new();
                map.insert(
                    "segment_id".into(),
                    Attribute {
                        tag: "segment_id".into(),
                        attribute_type: 'Z',
                        value: "001".into(),
                    },
                );
                map
            },
            ..Default::default()
        };

        let gtf = node.to_gtf(None)?;
        let gtf_str = gtf.to_str().unwrap();
        let lines: Vec<&str> = gtf_str.split('\n').collect();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("chr1\ttsg\texon\t100\t200\t.\t+\t.\texon_id \"001\""));
        assert!(lines[0].contains("segment_id \"001\""));
        assert!(lines[1].starts_with("chr1\ttsg\texon\t300\t400\t.\t+\t.\texon_id \"002\""));

        // Test with additional attributes
        let additional_attrs = vec![Attribute {
            tag: "transcript_id".into(),
            attribute_type: 'Z',
            value: "1".into(),
        }];

        let gtf_with_attrs = node.to_gtf(Some(&additional_attrs))?;
        let gtf_str = gtf_with_attrs.to_str().unwrap();
        let lines: Vec<&str> = gtf_str.split('\n').collect();

        assert!(lines[0].contains("transcript_id \"1\""));
        assert!(lines[1].contains("transcript_id \"1\""));

        Ok(())
    }
}
