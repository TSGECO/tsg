# Command-Line Help for `tsg`

This document contains the help content for the `tsg` command-line program.

**Command Overview:**

* [`tsg`↴](#tsg)
* [`tsg parse`↴](#tsg-parse)
* [`tsg fa`↴](#tsg-fa)
* [`tsg gtf`↴](#tsg-gtf)
* [`tsg vcf`↴](#tsg-vcf)
* [`tsg dot`↴](#tsg-dot)
* [`tsg json`↴](#tsg-json)
* [`tsg traverse`↴](#tsg-traverse)
* [`tsg merge`↴](#tsg-merge)
* [`tsg split`↴](#tsg-split)
* [`tsg query`↴](#tsg-query)

## `tsg`

Transcript Segment Graph (TSG) CLI tool

**Usage:** `tsg [OPTIONS] [COMMAND]`

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



## `tsg parse`

Parse a TSG file and validate its structure

**Usage:** `tsg parse <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path



## `tsg fa`

Convert a TSG file to FASTA format

**Usage:** `tsg fa [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output file path for the FASTA



## `tsg gtf`

Convert a TSG file to GTF format

**Usage:** `tsg gtf [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output file path for the GTF



## `tsg vcf`

Convert a TSG file to VCF format

**Usage:** `tsg vcf [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output file path for the VCF



## `tsg dot`

Convert a TSG file to DOT format for graph visualization

**Usage:** `tsg dot [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output DOT file path



## `tsg json`

Convert a TSG file to JSON format

**Usage:** `tsg json [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-p`, `--pretty` — Format JSON with indentation for better readability

  Default value: `false`
* `-o`, `--output <OUTPUT>` — Output file path for the JSON



## `tsg traverse`

Find and enumerate all valid paths through the graph

**Usage:** `tsg traverse [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-t`, `--text-path`

  Default value: `false`
* `-o`, `--output <OUTPUT>` — Output file path for the paths, default is stdout



## `tsg merge`

Merge multiple TSG files into a single TSG file

**Usage:** `tsg merge [OPTIONS] <INPUTS>...`

###### **Arguments:**

* `<INPUTS>` — Input TSG file paths

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output file path for the merged TSG



## `tsg split`

Split a TSG file into multiple TSG files

**Usage:** `tsg split [OPTIONS] <INPUT>`

###### **Arguments:**

* `<INPUT>` — Input TSG file path

###### **Options:**

* `-o`, `--output <OUTPUT>` — Output directory for the split TSG files



## `tsg query`

Query specific graphs from a TSG file

**Usage:** `tsg query [OPTIONS] --ids <IDS> <INPUT>`

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

