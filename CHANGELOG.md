# Changelog

All notable changes to this project will be documented in this file.

## [unreleased]

## [0.1.1](https://github.com/cauliyang/tsg/compare/v0.1.0...v0.1.1) - 2025-03-05

### Added

- Add completion generation feature for CLI
- *(cli)* Add option to output pretty JSON

### Other

- Update CLI documentation and examples
- Remove unnecessary import in edge module
- Update formatting and whitespace in test data files
- Update CLI command descriptions
- Update metadata attributes format in README and TSG files
- Update formatting in GTF and TSG test files and remove duplicate dependency in Cargo.toml
- Add JSON conversion functionality for TSG files, including new command and serialization methods
- Add to_json method for EdgeData struct and update test data formatting
- Remove unused import from graph.rs, update write calls to use writeln! in GTF and VCF, and adjust test data formatting
- Update GTF and VCF output formatting, add new Header struct, and modify test data
- Update test data formatting in GTF and VCF files
- Remove unused import of ParallelString from graph.rs
- Refactor TSGPath display implementation and update test data formatting
- Add changelog and configuration for git-cliff

### 🚀 Features

- Add Cargo.toml and source code files
- Update Cargo.toml and README.md with additional metadata and project description
- Implement graph module with read functionality and test case
- Add node module with interval and exon structures, and update graph read functionality
- Add initial implementation of edge module in graph
- Update node module to use BString for ID and add bstr dependency
- Update dependencies in Cargo.toml for improved performance and compatibility
- Add Rust toolchain and Clippy configuration, enhance graph structures with attributes and variants
- Enhance graph structure with new group and oriented element modules, and update edge and node definitions
- Remove unused write module, update dependencies, and enhance node and read modules with petgraph integration
- Enhance edge and group structures with builder pattern, improve attribute handling, and add unit tests for graph functionality
- Simplify edge creation by using default values in graph tests
- Update test data for graph structure with nodes, edges, chains, paths, sets, and attributes
- Add new ReadData struct and parsing functions
- Update graph node and edge lengths
- Enhance Header struct with builder pattern and update write logic for groups and attributes
- Update GitHub Actions workflow to use nightly toolchain and add .vscode to .gitignore; implement graph traversal and path representation
- Add section headers for improved readability in graph output and test data
- Add CLI tool for TSG file processing with parsing, DOT conversion, and path traversal commands
- Remove obsolete test output file to streamline project structure
- Enhance NodeData structure with reference ID and update parsing logic for TSG files
- Update NodeData structure to include strand information and adjust parsing logic for TSG files
- Update test data files and adjust metadata attributes for improved processing
- Refactor graph module by removing read functionality, adding I/O modules, and implementing GTF export for NodeData
- Remove obsolete test output file and clean up imports in node module
- Introduce path module and TSGPath struct for graph traversal, refactor graph.rs and update imports
- Implement CLI commands for TSG file processing and add DOT conversion functionality
- Add node and edge retrieval by index methods, implement GTF and VCF export stubs, and introduce FA module for sequence annotation
- Introduce Strand enum for DNA strand orientation and update NodeData to use Strand type
- Update DOT conversion to support custom node and edge labels, and integrate bon for builder patterns
- Add FASTA conversion command and functionality to CLI
- Add GTF conversion command and functionality to CLI
- Add VCF conversion command and functionality to CLI
- Enhance GTF output by including additional attributes in exon entries
- Add tests for Exons functionality and update test output file path
- Update JSON export function to return Result and modify test data for edge types
- Add unit tests for GTF and VCF output functions with sample data
- Add GitHub Actions workflow for Release-plz and update project description in Cargo.toml

### 🐛 Bug Fixes

- Correctly push EdgeIndexLabel to config for DOT conversion
- Remove unnecessary `into()` calls for node and edge IDs in graph module
- Correct formatting of exon entries in GTF test data

### 🚜 Refactor

- Update attribute handling in graph structures to use Vec<Attribute> instead of HashMap
- Simplify Header creation using builder pattern and clarify comment for reference start adjustment

### 📚 Documentation

- Update README to enhance documentation on TSG features, usage, and file format
- Update README to clarify read types for genomic location

### 🎨 Styling

- Improve enum variants comments
- Update toolchain to stable version and reorganize attributes
- Update project categories to reflect new additions
- Remove categories from Cargo.toml

### 🧪 Testing

- Add test for reading graph from file

### ⚙️ Miscellaneous Tasks

- Add CI workflow for Rust code with formatting, linting, and testing
- Update .gitignore, modify test data, and upgrade pre-commit dependencies

<!-- generated by git-cliff -->
