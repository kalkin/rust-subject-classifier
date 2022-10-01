# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1] - 2022-10-01

### Added

- Recognized Azure merge PRs

### Changed

- `Subject` derive `Clone`
- Make `Subject::scope()` `const` and return a reference
- Replace the refactor icon with 

### Fixed

- fix: Align Performance icon
- fix: Remove space after chore icon
- fix: Align refactor icon
- fix: Indentation of the refactor icon

## [0.4.0] - 2022-04-21

### Added

- Recognize commits starting with "move" as `Category::Rename`
- Add support for 'internal' conventional commit
- Recognize Bors GitHub App Merges

### Changed

- icon of refactor commit to `nf-mdi-recycle`
- icon of unrecognized commits to whitespace

## [0.3.0] - 2022-03-06

### Added

- Recognize Bitbucket PR merges
- Fix compilation errors in examples

### Changed

- Recognize subjects prefixed with the word “done”
- Recognize subjects prefixed with the word “feature”
- Recognize categories by the first word

## [0.2.0] - 2022-01-17

### Added

- Add documentation
- Add licensing information
- Add support for 'dev' conventional commit
- Add support for 'chore' conventional commit
- Recognize commits starting with issue or gi as Category::Issue
- Add support for Deprecate category
- Add support for Security type
- Icon for conventional commits breaking change
- Add documentation
- Add licensing information
- Add support for 'dev' conventional commit
- Add support for 'chore' conventional commit
- Recognize commits starting with issue or gi as Category::Issue
- Add support for Deprecate category
- Add support for Security type
- Icon for conventional commits breaking change

### Changed

- Make CONVENTIONAL_COMMIT_REGEX package private
- Category::Feat icon is 🎁 unicode wrapped present
- Recognize 'tests' type
- Prepend breaking changes text with '!'
- Make CONVENTIONAL_COMMIT_REGEX package private
- Category::Feat icon is 🎁 unicode wrapped present
- Recognize 'tests' type
- Prepend breaking changes text with '!'

### Fixed

- Adjust the present icon to be two cells big
- Add missing space to Category::Issue icon
- Add whitespace to Repo & Security icons
- Handling “breaking change”
- Adjust the present icon to be two cells big
- Add missing space to Category::Issue icon
- Add whitespace to Repo & Security icons
- Handling “breaking change”
