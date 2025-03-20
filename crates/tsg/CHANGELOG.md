# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3](https://github.com/cauliyang/tsg/compare/tsg-v0.1.2...tsg-v0.1.3) - 2025-03-20

### Added

- Add CHANGELOG files for tsg-cli, tsg-core, and tsg modules; update test_write.tsg data
- rename btsg module to tsg-btsg and add example for compression and decompression
- introduce tsg-core module with graph and I/O functionalities
- add BTSG module with compressor and decompressor examples
- Add method to parse TSG file from BufRead input
- add decompress_to_string method for BTSGDecompressor and update test files
- restructure project into a workspace with separate crates for TSG and TSG CLI

### Fixed

- Remove tsg-btsg crate and related examples; update dependencies in Cargo.toml

### Other

- update dependencies and improve reader handling in TSGraph
- simplify initialization of StringDictionary and update error handling in BTSGDecompressor
- update TSG format specification in documentation
