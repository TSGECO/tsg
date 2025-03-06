# Command-Line Help for `tsg`

This document contains the help content for the `tsg` command-line program.

**Command Overview:**

- [`tsg`↴](#tsg)
- [`tsg parse`↴](#tsg-parse)
- [`tsg fa`↴](#tsg-fa)
- [`tsg gtf`↴](#tsg-gtf)
- [`tsg vcf`↴](#tsg-vcf)
- [`tsg dot`↴](#tsg-dot)
- [`tsg json`↴](#tsg-json)
- [`tsg traverse`↴](#tsg-traverse)

## `tsg`

Command line interface for the TSG tool

**Usage:** `tsg [OPTIONS] [COMMAND]`

###### **Subcommands:**

- `parse` — Parse a TSG file and validate its structure
- `fa` — Convert a TSG file to FASTA format
- `gtf` — Convert a TSG file to GTF format
- `vcf` — Convert a TSG file to VCF format
- `dot` — Convert a TSG file to DOT format for graph visualization
- `json` — Convert a TSG file to JSON format
- `traverse` — Find and enumerate all valid paths through the graph

###### **Options:**

- `--generate <GENERATOR>`

  Possible values: `bash`, `elvish`, `fish`, `powershell`, `zsh`

- `-v`, `--verbose` — Sets the level of verbosity

## `tsg parse`

Parse a TSG file and validate its structure

**Usage:** `tsg parse <INPUT>`

###### **Arguments:**

- `<INPUT>` — Input TSG file path

## `tsg fa`

Convert a TSG file to FASTA format

**Usage:** `tsg fa [OPTIONS] --reference-genome <REFERENCE_GENOME> <INPUT>`

###### **Arguments:**

- `<INPUT>` — Input TSG file path

###### **Options:**

- `-r`, `--reference-genome <REFERENCE_GENOME>` — Path to the reference genome file
- `-o`, `--output <OUTPUT>` — Output file path for the FASTA

## `tsg gtf`

Convert a TSG file to GTF format

**Usage:** `tsg gtf [OPTIONS] <INPUT>`

###### **Arguments:**

- `<INPUT>` — Input TSG file path

###### **Options:**

- `-o`, `--output <OUTPUT>` — Output file path for the GTF

## `tsg vcf`

Convert a TSG file to VCF format

**Usage:** `tsg vcf [OPTIONS] <INPUT>`

###### **Arguments:**

- `<INPUT>` — Input TSG file path

###### **Options:**

- `-o`, `--output <OUTPUT>` — Output file path for the VCF

## `tsg dot`

Convert a TSG file to DOT format for graph visualization

**Usage:** `tsg dot [OPTIONS] <INPUT>`

###### **Arguments:**

- `<INPUT>` — Input TSG file path

###### **Options:**

- `-o`, `--output <OUTPUT>` — Output DOT file path

## `tsg json`

Convert a TSG file to JSON format

**Usage:** `tsg json [OPTIONS] <INPUT>`

###### **Arguments:**

- `<INPUT>` — Input TSG file path

###### **Options:**

- `-p`, `--pretty` — Format JSON with indentation for better readability

  Default value: `false`

- `-o`, `--output <OUTPUT>` — Output file path for the JSON

## `tsg traverse`

Find and enumerate all valid paths through the graph

**Usage:** `tsg traverse [OPTIONS] <INPUT>`

###### **Arguments:**

- `<INPUT>` — Input TSG file path

###### **Options:**

- `-o`, `--output <OUTPUT>` — Output file path for the paths
