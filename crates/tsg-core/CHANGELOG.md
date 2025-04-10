# Changelog

## [Unreleased]

## [0.1.8](https://github.com/TSGECO/tsg/compare/tsg-core-v0.1.7...tsg-core-v0.1.8)

### Added


- Enhance documentation for TSGraph to FA conversion and attribute representation - ([ce47d6b](https://github.com/TSGECO/tsg/commit/ce47d6b85a17a8b962d3787043b161a1e2b3dcbf))
- Update summary output description and correct chain references in test data - ([0774e06](https://github.com/TSGECO/tsg/commit/0774e06ac1e578e815d674c2e357dcab28a27e3f))

### Fixed


- Update summary output formatting and correct test data entries - ([3eea99c](https://github.com/TSGECO/tsg/commit/3eea99c83989b21e4c693f7d36fc9f97f36b7680))
- Correct formatting of chains in test_write.tsg - ([fd2b817](https://github.com/TSGECO/tsg/commit/fd2b817a766a09e819bf2492fea619f96a13db7c))

### Other


- Clean up whitespace and formatting in various files - ([fa803b5](https://github.com/TSGECO/tsg/commit/fa803b51b5818624bec304799998c8e8ec289bd1))


## [0.1.7](https://github.com/TSGECO/tsg/compare/tsg-core-v0.1.6...tsg-core-v0.1.7)

### Added


- Enhance TSGraph analysis output with additional headers and bubble detection logic; update test data for consistency - ([89887bb](https://github.com/TSGECO/tsg/commit/89887bb5d8c963661ab8d0b2bfe53808f7edb38c))
- Update GraphAnalysis trait methods to return Result types and modify bubble detection logic - ([06f98f3](https://github.com/TSGECO/tsg/commit/06f98f355a4629a457a544705253b0433929be25))
- Add tests for graph connectivity, cyclicity, and bubble detection; update NodeData struct with default values - ([692d282](https://github.com/TSGECO/tsg/commit/692d2829eca470c73ed896cda9c408987ef5255a))
- Add header command to CLI for printing TSG file headers - ([5843513](https://github.com/TSGECO/tsg/commit/58435131a0e02d9204c88621629dedbe224adbff))
- Enhance graph analysis with new traits and methods for path analysis - ([0456eea](https://github.com/TSGECO/tsg/commit/0456eeabcc2600d08c567011bd69ba57921ad90a))


## [0.1.6](https://github.com/cauliyang/tsg/compare/tsg-core-v0.1.5...tsg-core-v0.1.6)

### Added


- Add GraphAnalysis and PathAnalysis traits for future graph analysis functionality - ([5ce91f8](https://github.com/cauliyang/tsg/commit/5ce91f810c21e656bb39cda48e6955e277e72f38))
- Implement graph summary functionality and update test data for consistency - ([0cf710d](https://github.com/cauliyang/tsg/commit/0cf710d355384c289340e6e13110c355d7b0812c))
- Add summary command to CLI for generating TSG graph summaries - ([e7b74b3](https://github.com/cauliyang/tsg/commit/e7b74b3adac59169b6e4abda48459d00cd29245d))

### Fixed


- Reorder groups and transcripts for consistency in test_write.tsg - ([7f4fcea](https://github.com/cauliyang/tsg/commit/7f4fcea167753b21abad3e8d30888c230c5935d5))
- Update dependencies in Cargo.toml and reorder attributes in test files for consistency - ([b305692](https://github.com/cauliyang/tsg/commit/b30569254289c2f0bd895bd3760900c3f71851fe))
- Reorder attributes in test_write.tsg for consistency - ([ed115db](https://github.com/cauliyang/tsg/commit/ed115dbb74756e9156a9946ee170ac9d0a827f57))
- Optimize node capacity calculation and improve trait documentation - ([90a1483](https://github.com/cauliyang/tsg/commit/90a148383fb2ed324e99c00d9f7ae038f1c37cb7))
- Update pre-commit hooks and enhance Makefile with additional commands - ([2c928a4](https://github.com/cauliyang/tsg/commit/2c928a470bd1c1913fa46c7a58d0bdcddd2a3232))
- Reorder attributes and groups in test files for consistency - ([799d413](https://github.com/cauliyang/tsg/commit/799d413b37dd7af2739ba4ddba2a3eacda2ba4a2))

### Other


- *(test.gtf, test_write.tsg)* Reorder attributes by type and name for better readability - ([0d17669](https://github.com/cauliyang/tsg/commit/0d176694be091a4e0402f5dc93fb968afdb4141f))
- Add TODO comment for future use of the GraphAnalysis module - ([aca1160](https://github.com/cauliyang/tsg/commit/aca1160360fce61c1ba15566da6a2a1dfec6e90d))


## [0.1.3](https://github.com/cauliyang/tsg/compare/tsg-core-v0.1.2...tsg-core-v0.1.3)

### Added


- Add CHANGELOG files for tsg-cli, tsg-core, and tsg modules; update test_write.tsg data - ([d066a68](https://github.com/cauliyang/tsg/commit/d066a68abd045fc6560ba4a631e898610ec30728))
- Enhance Interval and Exons structs with detailed documentation and new methods - ([4771032](https://github.com/cauliyang/tsg/commit/477103247dca208ebcf47a26db379e86bedae112))

### Fixed


- Add utils module to graph and update hash identifier example in documentation - ([dd3dac4](https://github.com/cauliyang/tsg/commit/dd3dac45a9bc079d8f615e40992481df31581783))
- Remove tsg-btsg crate and related examples; update dependencies in Cargo.toml - ([c854328](https://github.com/cauliyang/tsg/commit/c854328d3f08b6098b2068f0032ccc5b308518e3))
- Update .gitignore and rename tsg binary to tsg-cli; simplify node addition in graph - ([5a1360a](https://github.com/cauliyang/tsg/commit/5a1360af4b77f4e9782252566247bb2bc4af0d2a))
- Update genomic location format in node data and improve test data consistency - ([62b76f8](https://github.com/cauliyang/tsg/commit/62b76f8f47e93de39aeddabdf687b7b8dfefce0e))
- Correct formatting inconsistencies in test_write.tsg - ([e03d24f](https://github.com/cauliyang/tsg/commit/e03d24f8da6e57b614aa8e9477f672d1beab0a91))
- Improve graph block handling and clean up test data formatting - ([276706e](https://github.com/cauliyang/tsg/commit/276706e1a7b27e0657e8d68ac06ee3d559233bbb))
- Clean up whitespace and update comments in BTSG and test files - ([c739699](https://github.com/cauliyang/tsg/commit/c73969962e72ccb62cb325bdd1ccec8c8636aa6a))
- Add test for NodeData parsing and clean up whitespace in from_str method - ([d725b7e](https://github.com/cauliyang/tsg/commit/d725b7ed1993c09b1e433c638a292a2c9cfdba75))
- Update graph node handling to create placeholder nodes if not found - ([c7a7b29](https://github.com/cauliyang/tsg/commit/c7a7b29a73584292db44c46a609e18f5c3acb0e3))
- Reorder zstd import and remove unnecessary blank line in GraphAnalysis trait - ([d1e69a1](https://github.com/cauliyang/tsg/commit/d1e69a1fc5de62aa85015286ca2abc7b388b6205))

### Other


- Remove unnecessary whitespace and comments - ([8d9c56f](https://github.com/cauliyang/tsg/commit/8d9c56f6bd8b5f67891f0a7b28f5166b053f60d1))
- Update pre-commit hooks and improve CLI documentation; refactor function names for clarity - ([d289043](https://github.com/cauliyang/tsg/commit/d2890439a0477bf6126b483286d12befcc550f2a))
