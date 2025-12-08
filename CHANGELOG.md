# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.3.1) - 2025-12-08

### Performance

- avoid using fmt machinery in hot paths, instead pushing directly

## [0.3.0](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.3.0) - 2025-12-08

### Added

- axolotl logo in version screen
- consistent key ordering

### Deprecated

- removed help subcommand

### Documentation

- autogenerate examples and add examples to subcommands
- update readme with correct command
- add xtask to generate readmes

### Performance

- TokenStream iterator instead of collecting into intermediary Vec


## [0.2.2](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.2.2) - 2025-12-07

### Documentation

- add mise installer steps
- update readme with new command format and installation instructions. removes extra notes

### Performance

- join_into utility to declaritively avoid allocating delimiter strings
- write to single buffer instead of allocating buffer per JSON value
- don't use anstream for content without ansi

## [0.2.0](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.2.0) - 2025-12-06

### Added

- subcommands - check and format with uglify flag


## [0.1.5](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.1.5) - 2025-12-05

Test for publishing flow


## [0.1.4](https://github.com/20jasper/JJPWRGEM/releases/tag/jjpwrgem-v0.1.4) - 2025-12-05

### Feature
- pretty format JSON
- error messages on failure

