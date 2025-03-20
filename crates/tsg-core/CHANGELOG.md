# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3](https://github.com/cauliyang/tsg/compare/tsg-core-v0.1.2...tsg-core-v0.1.3) - 2025-03-20

### Added

- Add CHANGELOG files for tsg-cli, tsg-core, and tsg modules; update test_write.tsg data
- Enhance Interval and Exons structs with detailed documentation and new methods

### Fixed

- Add utils module to graph and update hash identifier example in documentation
- Remove tsg-btsg crate and related examples; update dependencies in Cargo.toml
- Update .gitignore and rename tsg binary to tsg-cli; simplify node addition in graph
- Update genomic location format in node data and improve test data consistency
- Correct formatting inconsistencies in test_write.tsg
- Improve graph block handling and clean up test data formatting
- Clean up whitespace and update comments in BTSG and test files
- add test for NodeData parsing and clean up whitespace in from_str method
- update graph node handling to create placeholder nodes if not found
- reorder zstd import and remove unnecessary blank line in GraphAnalysis trait

### Other

- Remove unnecessary whitespace and comments
- update pre-commit hooks and improve CLI documentation; refactor function names for clarity
