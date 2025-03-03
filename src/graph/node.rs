use std::fmt;
use std::str::FromStr;

use crate::graph::Attribute;
use ahash::HashMap;
use anyhow::Context;
use bstr::BString;
use derive_builder::Builder;
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
pub struct ReadData {
    #[builder(setter(into))]
    pub id: BString,
    #[builder(setter(into))]
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
    SO,
    IN,
    SI,
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

/// Node in the transcript segment graph
#[derive(Debug, Clone, Default, Builder)]
pub struct NodeData {
    #[builder(setter(into))]
    pub id: BString,
    #[builder(setter(into))]
    pub reference_id: BString,
    pub strand: char,
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
        // N  <id>  <exons>  <reads>  [<seq>]

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
        let strand = reference_and_exons[1].chars().next().unwrap();
        let exons = reference_and_exons[2].parse()?;

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
