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

    pub fn to_gtf(&self, attributes: Option<&[Attribute]>) -> Result<BString> {
        // chr1    scannls exon    173867960       173867991       .       -       .       exon_id "001"; mega_exon_id "0001"; ptc "1"; ptf "1.0"; transcript_id "3x1"; gene_id "3";
        let mut res = vec![];
        for (idx, exon) in self.exons.exons.iter().enumerate() {
            let mut gtf = String::from("");
            gtf.push_str(self.reference_id.to_str().unwrap());
            gtf.push_str("\ttsg\texon\t");
            gtf.push_str(&format!("{}\t{}\t", exon.start, exon.end));
            gtf.push_str(".\t");
            gtf.push_str(self.strand.to_string().as_str());
            gtf.push_str("\t.\t");
            gtf.push_str(format!("exon_id \"{:03}\"; ", idx).as_str());

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

        let fields: Vec<&str> = s.split('\t').collect();
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
