# Changelog

## [Unreleased]

## [0.1.3](https://github.com/cauliyang/tsg/compare/tsg-v0.1.2...tsg-v0.1.3)

### Added


- Add CHANGELOG files for tsg-cli, tsg-core, and tsg modules; update test_write.tsg data - ([d066a68](https://github.com/cauliyang/tsg/commit/d066a68abd045fc6560ba4a631e898610ec30728))
- Rename btsg module to tsg-btsg and add example for compression and decompression - ([b8c2386](https://github.com/cauliyang/tsg/commit/b8c2386dfba3893ef51ac74ab0efa24d5a6e69f6))
- Introduce tsg-core module with graph and I/O functionalities - ([9d95df1](https://github.com/cauliyang/tsg/commit/9d95df14876841bac9cd53fc1980f0b7b1e43ffa))
- Add BTSG module with compressor and decompressor examples - ([2950a07](https://github.com/cauliyang/tsg/commit/2950a07c06163ef1bab3cd2545ae3e7bcb9c0fe5))
- Add method to parse TSG file from BufRead input - ([33e2c8a](https://github.com/cauliyang/tsg/commit/33e2c8abb415acd90878b9cc30fff2b538c18f8f))
- Add decompress_to_string method for BTSGDecompressor and update test files - ([0ad8646](https://github.com/cauliyang/tsg/commit/0ad86469deb3b1142d6fe02f84e2251b6847103f))
- Restructure project into a workspace with separate crates for TSG and TSG CLI - ([5da3b23](https://github.com/cauliyang/tsg/commit/5da3b23e04bf7289c86e8104a5b6df920ae5f87f))

### Fixed


- Remove tsg-btsg crate and related examples; update dependencies in Cargo.toml - ([c854328](https://github.com/cauliyang/tsg/commit/c854328d3f08b6098b2068f0032ccc5b308518e3))

### Other


- Update dependencies and improve reader handling in TSGraph - ([a6f55fa](https://github.com/cauliyang/tsg/commit/a6f55fa342e35db82d18739b86ffdecbbdc2c97c))
- Simplify initialization of StringDictionary and update error handling in BTSGDecompressor - ([c37479e](https://github.com/cauliyang/tsg/commit/c37479e6f9c1839ad66e65e594880ac80b5557d8))
- Update TSG format specification in documentation - ([5ae674b](https://github.com/cauliyang/tsg/commit/5ae674bfd3ff78d2f854b5a60c15ed4a01dae975))
