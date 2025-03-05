use std::path::Path;

use crate::graph::TSGraph;
use anyhow::Result;
use std::io::Write;

static VCF_HEADER: &[&str] = &[
    "##fileformat=VCFv4.3",
    "##source=tsg",
    "##INFO=<ID=CANONICAL,Number=0,Type=Flag,Description=\"Canonical splice site\">",
    "##INFO=<ID=NONCANONICAL,Number=0,Type=Flag,Description=\"Noncanonical splice site\">",
    "##INFO=<ID=BOUNDARY,Number=1,Type=String,Description=\"The coding exon boundary type of event, BOTH, LEFT, RIGHT, NEITHER.\">",
    "##INFO=<ID=DP1,Number=1,Type=Integer,Description=\"Total read depth at the breakpoint1\">",
    "##INFO=<ID=DP2,Number=1,Type=Integer,Description=\"Total read depth at the breakpoint2\">",
    "##INFO=<ID=SR,Number=1,Type=Integer,Description=\"The number of support reads for the breakpoints\">",
    "##INFO=<ID=OSR,Number=1,Type=Integer,Description=\"The number of support reads for the breakpoints before rescuer\">",
    "##INFO=<ID=PSI,Number=1,Type=Float,Description=\"Estimated Percent splice-in in the range (0,1], representing the percentage of NLS transcripts\">",
    "##INFO=<ID=SVMETHOD,Number=1,Type=String,Description=\"Type of approach used to detect SV\">",
    "##INFO=<ID=SVTYPE,Number=1,Type=String,Description=\"The type of event, DEL, TDUP, IDUP, INV, TRA.\">",
    "##INFO=<ID=SVLEN,Number=1,Type=Integer,Description=\"Difference in length between REF and ALT alleles\">",
    "##INFO=<ID=CHR2,Number=1,Type=String,Description=\"Chromosome for END coordinate in case of a translocation\">",
    "##INFO=<ID=SVEND,Number=1,Type=Integer,Description=\"2nd position of the structural variant\">",
    "##INFO=<ID=END,Number=1,Type=Integer,Description=\"A placeholder for END coordinate in case of a translocation\">",
    "##INFO=<ID=STRAND1,Number=1,Type=String,Description=\"Strand for breakpoint1\">",
    "##INFO=<ID=STRAND2,Number=1,Type=String,Description=\"Strand for breakpoint2\">",
    "##INFO=<ID=MODE1,Number=1,Type=String,Description=\"Mode for softclipped reads at breakpoint1\">",
    "##INFO=<ID=MODE2,Number=1,Type=String,Description=\"Mode for softclipped reads at breakpoint2\">",
    "##INFO=<ID=GENE1,Number=1,Type=String,Description=\"Overlapped coding gene for breakpoint1\">",
    "##INFO=<ID=GENE2,Number=1,Type=String,Description=\"Overlapped coding gene for breakpoint2\">",
    "##INFO=<ID=MEGAEXON1,Number=.,Type=String,Description=\"ID for source mega exon\">",
    "##INFO=<ID=MEGAEXON2,Number=.,Type=String,Description=\"ID for target mega exon\">",
    "##INFO=<ID=HOMSEQ,Number=1,Type=String,Description=\"MicroHomology sequence\">",
    "##INFO=<ID=INSSEQ,Number=1,Type=String,Description=\"MicroInsertion sequence\">",
    "##INFO=<ID=TRANSCRIPT_ID,Number=.,Type=String,Description=\"Transcript ID\">",
    "##INFO=<ID=GENE_ID,Number=1,Type=String,Description=\"Gene ID\">",
    "##INFO=<ID=SR_ID,Number=.,Type=String,Description=\"Support read ID\">",
    "##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">",
    "##ALT=<ID=DEL,Description=\"Deletion\">",
    "##ALT=<ID=TDUP,Description=\"Tandem duplication\">",
    "##ALT=<ID=IDUP,Description=\"Inverted duplication\">",
    "##ALT=<ID=INV,Description=\"Inversion\">",
    "##ALT=<ID=TRA,Description=\"Translocation\">",
    "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT",
];

pub fn to_vcf<P: AsRef<Path>>(tsg_graph: &TSGraph, output: P) -> Result<()> {
    let paths = tsg_graph.traverse()?;
    let output_file = std::fs::File::create(output)?;
    let mut writer = std::io::BufWriter::new(output_file);

    let header = VCF_HEADER;

    for line in header {
        writeln!(writer, "{}", line)?;
    }

    for path in paths {
        let seq = path.to_vcf()?;
        writeln!(writer, "{}", seq)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_vcf() {
        let tsg_graph = TSGraph::from_file("tests/data/test.tsg").unwrap();
        let output = "tests/data/test.vcf";
        to_vcf(&tsg_graph, output).unwrap();
    }
}
