# Changelog

All notable changes to this project will be documented in this file.

## [0.15.0] - 2022-07-25

### Added
- function --> func
- can use `use * from id`; This searches deps for `id`. Currently only top level paths.
- can use `impl`s, which create's a corresponding `resource` with static methods and instance methods with `@mutable` for methods with `&mut self`
- Added `aha-wit-parser` version `0.2.0`, to ensure generated wit is valid and will resolve deps used in `use`.

---
## [0.14.0] - 2022-06-14
### Added
### Changed
- f32/f64 --> float32/float64. (Not sure why this decision was made. Why make it longer?)
- Multi-value returns are now tuples
- Can no longer have recursive type definitions.
---
## [0.12.0] - 2022-04-09
### Added
- `Wit` type [PR #25](https://github.com/bnjjj/witgen/pull/25)
- `Witgen` struct; expose `cargo-witgen` as a library; and add new arg `input_dir` [#PR 26](https://github.com/bnjjj/witgen/pull/25)
### Changed
 - Use `syn-file-expand` instead of Cargo [PR #25](https://github.com/bnjjj/witgen/pull/25)
---
## [0.11.0] - 2022-03-10
### Added
### Changed
### Fixed
- Fix enums with named fields and add initial suite of tests(thanks [@willemneal](https://github.com/willemneal)) [PR #22](https://github.com/bnjjj/witgen/pull/22)
### Removed

---
## [0.10.0] - 2022-03-09
### Added
- Add support for named fields in variants for enums (thanks [@willemneal](https://github.com/willemneal)) [PR #21](https://github.com/bnjjj/witgen/pull/21)
### Changed
### Fixed
### Removed

---
## [0.9.0] - 2022-02-25
### Added
- Add support of wit enum (thanks [@willemneal](https://github.com/willemneal))
### Changed
### Fixed
### Removed

---
## [0.8.0] - 2022-02-09
### Added
- Add support of reference type (thanks [@willemneal](https://github.com/willemneal))
### Changed
### Fixed
### Removed

---
## [0.7.0] - 2022-01-27
### Added
- Add 2 CLI options for adding to top of file `--prefix-file` and `--prefix-string` (thanks [@willemneal](https://github.com/willemneal))
### Changed
### Fixed
### Removed

---
## [0.6.0] - 2022-01-26
### Added
- Add the support of `HashMap` (thanks [@willemneal](https://github.com/willemneal))
### Changed
- Update the structure of crates, you now can use generator functions with the `witgen_macro_helper` crate
### Fixed
### Removed

---
## [0.5.0] - 2021-12-23
### Added
### Changed
- Update the naming convention adopted by [wit-bindgen](https://github.com/bytecodealliance/wit-bindgen/pull/119) to generate only kebab-case declarations in `wit` files
### Fixed
### Removed
