# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2024-05-04

### 🚀 Features

- Impl `IntoIterator` for `&Poly` and `&mut Poly`

### 🐛 Bug Fixes

- [**breaking**] Clippy warnings

### 📚 Documentation

- Added developer documentation

### ⚙️ Miscellaneous Tasks

- Update itertools to 0.12.1
- Updated TODOs
- Added commitlint to pre-commit hooks
- Added git-cliff dev tool
- Added old changes to CHANGELOG.md

## [0.1.13] - 2023-11-01

### Changes
- Add Bessel polynomials
- Add revere bessel polynomials

### Bug Fixes
- `Poly::term()` having incorrect coefficients

## [0.1.12] - 2023-09-08

### Changes

- Div_rem now returns anyhow::Result
- Switched from num_complex and num_traits to num crate
- Div_rem takes ownership of both arguments
- Reduced cloning in ops
- More pre-computed chebyshev polynomials
- Div for poly and complex
- More combinations of borrowed and owned variants of operators
- More combinations of borrowed and owned operators
- Almost_zero method
- Checked_div and checked_rem
- CheckedDiv trait
- All combinations of owned and borrowed ops
- Missing cargo.lock
- Checked and negative indexing
- More concise macro syntax for complex values with no imaginary component

<!-- generated by git-cliff -->