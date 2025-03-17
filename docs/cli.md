# Command-Line Help for `tsg-cli`

This document contains the help content for the `tsg-cli` command-line program.

**Command Overview:**

* [`tsg-cli`↴](#tsg-cli)
* [`tsg-cli parse`↴](#tsg-cli-parse)
* [`tsg-cli fa`↴](#tsg-cli-fa)
* [`tsg-cli gtf`↴](#tsg-cli-gtf)
* [`tsg-cli vcf`↴](#tsg-cli-vcf)
* [`tsg-cli dot`↴](#tsg-cli-dot)
* [`tsg-cli json`↴](#tsg-cli-json)
* [`tsg-cli traverse`↴](#tsg-cli-traverse)
* [`tsg-cli merge`↴](#tsg-cli-merge)
* [`tsg-cli split`↴](#tsg-cli-split)
* [`tsg-cli query`↴](#tsg-cli-query)

## `tsg-cli`

Transcript Segment Graph (TSG) CLI tool

**Usage:** `tsg-cli [OPTIONS] [COMMAND]`

###### **Subcommands:**

* `parse` — Parse a TSG file and validate its structure
* `fa` — Convert a TSG file to FASTA format
* `gtf` — Convert a TSG file to GTF format
* `vcf` — Convert a TSG file to VCF format
* `dot` — Convert a TSG file to DOT format for graph visualization
* `json` — Convert a TSG file to JSON format
* `traverse` — Find and enumerate all valid paths through the graph
* `merge` — Merge multiple TSG files into a single TSG file
* `split` — Split a TSG file into multiple TSG files
* `query` — Query specific graphs from a TSG file

###### **Options:**

* `--generate <GENERATOR>`

  Possible values: `bash`, `elvish`, `fish`, `powershell`, `zsh`

* `-v`, `--verbose` — Increase logging verbosity
* `-q`, `--quiet` — Decrease logging verbosity



## `tsg-cli parse`

Parse a TSG file and validate its structure

**Usage:** `tsg-cli parse <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path



## `tsg-cli fa`

Convert a TSG file to FASTA format

**Usage:** `tsg-cli fa [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output file path for the FASTA



## `tsg-cli gtf`

Convert a TSG file to GTF format

**Usage:** `tsg-cli gtf [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output file path for the GTF



## `tsg-cli vcf`

Convert a TSG file to VCF format

**Usage:** `tsg-cli vcf [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output file path for the VCF



## `tsg-cli dot`

Convert a TSG file to DOT format for graph visualization

**Usage:** `tsg-cli dot [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output DOT file path



## `tsg-cli json`

Convert a TSG file to JSON format

**Usage:** `tsg-cli json [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-p`, `--pretty` — Format JSON with indentation for better readability

  Default value: `false`
* `-o`, `--output <OUTPUT>` — Output file path for the JSON



## `tsg-cli traverse`

Find and enumerate all valid paths through the graph

**Usage:** `tsg-cli traverse [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-t`, `--text-path`

  Default value: `false`
* `-o`, `--output <OUTPUT>` — Output file path for the paths, default is stdout



## `tsg-cli merge`

Merge multiple TSG files into a single TSG file

**Usage:** `tsg-cli merge [OPTIONS] <INPUTS>...`

###### **Arguments:**

* `<INPUTS>` — Input TSG file paths

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output file path for the merged TSG



## `tsg-cli split`

Split a TSG file into multiple TSG files

**Usage:** `tsg-cli split [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output directory for the split TSG files



## `tsg-cli query`

Query specific graphs from a TSG file

**Usage:** `tsg-cli query [OPTIONS] --ids <IDS> <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-i`, `--ids <IDS>` — Graph IDs to query, can be separated by commas
* `--ids-file <IDS_FILE>` — File containing graph IDs to query (one per line)
* `-o`, `--output <OUTPUT>` — Output file path for the queried graphs



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

