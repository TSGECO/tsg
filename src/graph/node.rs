use std::str::FromStr;

use anyhow::Result;
use anyhow::anyhow;

// Define the interval struct
// []
#[derive(Debug)]
pub struct interval {
    pub start: usize,
    pub end: usize,
}

impl FromStr for interval {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.split('-');
        let start = parts.next().ok_or_else(|| anyhow!("Missing start"))?;
        let end = parts.next().ok_or_else(|| anyhow!("Missing end"))?;
        Ok(interval {
            start: start.parse()?,
            end: end.parse()?,
        })
    }
}

#[derive(Debug)]
pub struct Exons {
    pub exons: Vec<interval>,
}

impl FromStr for Exons {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let exons = s
            .split(',')
            .map(|x| x.parse())
            .collect::<Result<Vec<interval>>>()?;
        Ok(Exons { exons })
    }
}

impl Exons {
    pub fn introns(&self) -> Vec<interval> {
        let mut introns = Vec::new();
        for i in 0..self.exons.len() - 1 {
            introns.push(interval {
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

pub struct NodeAttributes {
    pub id: String,
    pub exons: Exons,
}
