# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.3](https://github.com/cauliyang/tsg/compare/tsg-cli-v0.1.2...tsg-cli-v0.1.3) - 2025-03-20

### Added

- Add CHANGELOG files for tsg-cli, tsg-core, and tsg modules; update test_write.tsg data
- introduce tsg-core module with graph and I/O functionalities
- restructure project into a workspace with separate crates for TSG and TSG CLI
- Add support for multiple graphs within a single file
- Add clap-markdown support and tsg-cli changes
- Add logo to README and refine TSG logo
- Update Cargo.toml and README.md with additional metadata and project description

### Fixed

- Update .gitignore and rename tsg binary to tsg-cli; simplify node addition in graph

### Other

- Update README.md
- Update metadata attributes format in README and TSG files
- Update README to clarify read types for genomic location
- Update README to enhance documentation on TSG features, usage, and file format
- Initial commit
