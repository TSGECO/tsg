use std::str::FromStr;

use crate::graph::Attribute;
use ahash::HashMap;
use bstr::BString;
use derive_builder::Builder;
use std::io;

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

#[derive(Debug, Builder, Clone)]
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
}

/// Node in the transcript segment graph
#[derive(Debug, Clone)]
pub struct Node {
    pub id: String,
    pub exons: Exons,
    pub reads: Vec<BString>,
    pub sequence: Option<String>,
    pub attributes: HashMap<String, Attribute>,
}
