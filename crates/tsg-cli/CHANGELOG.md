# Changelog

## [Unreleased]

## [0.1.9](https://github.com/TSGECO/tsg/compare/tsg-cli-v0.1.8...tsg-cli-v0.1.9)

### Added


- Add documentation for to_gtf and to_vcf functions to clarify usage and parameters - ([a72c0f4](https://github.com/TSGECO/tsg/commit/a72c0f420ff9bf606ac3a1929bd6e9a4a25c1dad))

### Fixed


- Restore traverse module usage in CLI; ensure proper command functionality - ([23d63b8](https://github.com/TSGECO/tsg/commit/23d63b8287ce934fe546ee23ecd215b63c65714e))
- Update Makefile to use pdflatex for PDF generation; refactor README for clarity and consistency - ([f620d6a](https://github.com/TSGECO/tsg/commit/f620d6a50494f8c9a68ea3f76010defc60770a17))
- Refactor EdgeData and InterGraphLink to use builder pattern; update test data for consistency - ([cdc4533](https://github.com/TSGECO/tsg/commit/cdc4533be1e2f382e09213b2c0681760d2a15434))

### Other


- Remove unused path module; add traverse functionality to output graph paths - ([ed20cf2](https://github.com/TSGECO/tsg/commit/ed20cf2579eba3a7dc9bdcc05bf66a7018e22414))


## [0.1.8](https://github.com/TSGECO/tsg/compare/tsg-cli-v0.1.7...tsg-cli-v0.1.8)

### Added


- Enhance documentation for TSGraph to FA conversion and attribute representation - ([ce47d6b](https://github.com/TSGECO/tsg/commit/ce47d6b85a17a8b962d3787043b161a1e2b3dcbf))
- Update summary output description and correct chain references in test data - ([0774e06](https://github.com/TSGECO/tsg/commit/0774e06ac1e578e815d674c2e357dcab28a27e3f))

### Fixed


- Update summary output formatting and correct test data entries - ([3eea99c](https://github.com/TSGECO/tsg/commit/3eea99c83989b21e4c693f7d36fc9f97f36b7680))

### Other


- Clean up whitespace and formatting in various files - ([fa803b5](https://github.com/TSGECO/tsg/commit/fa803b51b5818624bec304799998c8e8ec289bd1))


## [0.1.6](https://github.com/cauliyang/tsg/compare/tsg-cli-v0.1.5...tsg-cli-v0.1.6)

### Added


- Implement graph summary functionality and update test data for consistency - ([0cf710d](https://github.com/cauliyang/tsg/commit/0cf710d355384c289340e6e13110c355d7b0812c))
- Add summary command to CLI for generating TSG graph summaries - ([e7b74b3](https://github.com/cauliyang/tsg/commit/e7b74b3adac59169b6e4abda48459d00cd29245d))


## [0.1.3](https://github.com/cauliyang/tsg/compare/tsg-cli-v0.1.2...tsg-cli-v0.1.3)

### Added


- Add CHANGELOG files for tsg-cli, tsg-core, and tsg modules; update test_write.tsg data - ([d066a68](https://github.com/cauliyang/tsg/commit/d066a68abd045fc6560ba4a631e898610ec30728))
- Introduce tsg-core module with graph and I/O functionalities - ([9d95df1](https://github.com/cauliyang/tsg/commit/9d95df14876841bac9cd53fc1980f0b7b1e43ffa))
- Restructure project into a workspace with separate crates for TSG and TSG CLI - ([5da3b23](https://github.com/cauliyang/tsg/commit/5da3b23e04bf7289c86e8104a5b6df920ae5f87f))
- Add support for multiple graphs within a single file - ([51cf8fc](https://github.com/cauliyang/tsg/commit/51cf8fc8dcefba804a7e066e7002b0e92bb4f8dc))
- Add clap-markdown support and tsg-cli changes - ([f7ccd3c](https://github.com/cauliyang/tsg/commit/f7ccd3cd12925f3fa77de451cfc65cd0990aefef))
- Add logo to README and refine TSG logo - ([f76f433](https://github.com/cauliyang/tsg/commit/f76f4339edf9d6d1963078f5f713287277e67e5a))
- Update Cargo.toml and README.md with additional metadata and project description - ([dd59ae3](https://github.com/cauliyang/tsg/commit/dd59ae390a96e6dd820c03dd3cd7a4cedd892297))

### Fixed


- Update .gitignore and rename tsg binary to tsg-cli; simplify node addition in graph - ([5a1360a](https://github.com/cauliyang/tsg/commit/5a1360af4b77f4e9782252566247bb2bc4af0d2a))

### Other


- Update changelog template in release-plz.toml - ([d086ad6](https://github.com/cauliyang/tsg/commit/d086ad65149586cb7b9df3d527a36bcd040e42c9))
- Update README.md - ([a4408b0](https://github.com/cauliyang/tsg/commit/a4408b055a3ad9c950dbd27e71d27057b63671d2))
- Update metadata attributes format in README and TSG files - ([d4bfbd6](https://github.com/cauliyang/tsg/commit/d4bfbd60a9bf189819c19a059b6406d1d70b3840))
- Update README to clarify read types for genomic location - ([b0a44b6](https://github.com/cauliyang/tsg/commit/b0a44b6c797f67f230580ffc35aa516bb7981f7c))
- Update README to enhance documentation on TSG features, usage, and file format - ([6207da9](https://github.com/cauliyang/tsg/commit/6207da9f56a9082d3844ad8e6feb1d300b0bbbd0))
- Initial commit - ([2677300](https://github.com/cauliyang/tsg/commit/26773001f4da0e3d28788005a429dc3030d6c0c5))
