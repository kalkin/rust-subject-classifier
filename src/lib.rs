// Copyright (c) 2022 Bahtiar `kalkin` Gadimov <bahtiar@gadimov.de>
//
// This file is part of subject-classifier.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! Library for classifying a commit by it's subject. Tries hard to recognize the subject type
//! according to the commit message. Supports [Conventional Commits Standard v1.0.0](https://www.conventionalcommits.org/en/v1.0.0)
//!
//! ```rust
//! use subject_classifier::Subject;
//!
//! let subject = subject_classifier::Subject::from("feat(Stuff): Add a new feature XYZ");
//! println!("Icon: {}, scope {:?}, msg: {}",
//!         subject.icon(),
//!         subject.scope(),
//!         subject.description());
//! ```
use lazy_static::lazy_static;
use regex::{Captures, Regex, RegexBuilder};

lazy_static! {
    static ref CONVENTIONAL_COMMIT_REGEX: Regex =
        Regex::new(r"(?i)^(SECURITY FIX!?|BREAKING CHANGE!?|\w+!?)(\(.+\)!?)?[/:\s]*(.+)")
            .expect("Valid RegEx");
    static ref UPDATE_REGEX: Regex = Regex::new(r#"^Update :?(.+) to (.+)"#).expect("Valid RegEx");
    static ref SPLIT_REGEX: Regex =
        Regex::new(r#"^Split '(.+)/' into commit '(.+)'"#).expect("Valid RegEx");
    static ref IMPORT_REGEX: Regex = Regex::new(r#"^:?(.+) Import .+⸪(.+)"#).expect("Valid RegEx");
    static ref RELEASE_REGEX1: Regex =
        RegexBuilder::new(r#"^(?:Release|Bump) :?(.+)@v?([0-9.]+)\b.*"#)
            .case_insensitive(true)
            .build()
            .expect("Valid Regex");
    static ref RELEASE_REGEX2: Regex = RegexBuilder::new(r#"^(?:Release|Bump)\s.*?v?([0-9.]+).*"#)
        .case_insensitive(true)
        .build()
        .expect("Valid Regex");
    static ref PR_REGEX: Regex =
        Regex::new(r"^Merge (?:remote-tracking branch '.+/pr/(\d+)'|pull request #(\d+) from .+)$")
            .expect("Valid RegEx");
    static ref PR_REGEX_BB: Regex =
        Regex::new(r"^Merge pull request #(\d+) in .+ from .+$").expect("Valid RegEx");
    static ref ADD_REGEX: Regex = Regex::new(r"(?i)^add:?\s*").expect("Valid RegEx");
    static ref FIX_REGEX: Regex =
        Regex::new(r"(?i)^(bug)?fix(ing|ed)?(\(.+\))?[/:\s]+").expect("Valid Regex");
}

/// Represents different subtree operations encoded in the commit message.
#[derive(Debug, Eq, PartialEq)]
pub enum SubtreeOperation {
    Import { subtree: String, git_ref: String },
    Split { subtree: String, git_ref: String },
    Update { subtree: String, git_ref: String },
}

/// The type of the commit
#[derive(Debug, Eq, PartialEq)]
pub enum Type {
    Archive,
    Build,
    Change,
    Chore,
    Ci,
    Dev,
    Deps,
    Docs,
    Deprecate,
    Feat,
    Fix,
    I18n,
    Issue,
    Improvement,
    Other,
    Perf,
    Refactor,
    Repo,
    Security,
    Style,
    Test,
}
/// Classified subject
///
/// ```rust
/// use subject_classifier::Subject;
///
/// let subject = Subject::from("feat: Some new feature");
/// ```
#[derive(Debug, Eq, PartialEq)]
pub enum Subject {
    ConventionalCommit {
        breaking_change: bool,
        category: Type,
        scope: Option<String>,
        description: String,
    },
    Fixup(String),
    PullRequest {
        id: String,
        description: String,
    },
    Release {
        version: String,
        scope: Option<String>,
        description: String,
    },
    Remove(String),
    Rename(String),
    Revert(String),
    SubtreeCommit {
        operation: SubtreeOperation,
        description: String,
    },
    Simple(String),
}
//
impl From<&str> for Subject {
    fn from(subject: &str) -> Self {
        #[allow(clippy::option_if_let_else)]
        if let Some(caps) = RELEASE_REGEX1.captures(subject) {
            Self::Release {
                version: caps[2].to_string(),
                scope: Some(caps[1].to_string()),
                description: subject.to_string(),
            }
        } else if let Some(caps) = RELEASE_REGEX2.captures(subject) {
            Self::Release {
                version: caps[1].to_string(),
                scope: None,
                description: subject.to_string(),
            }
        } else if let Some(caps) = PR_REGEX.captures(subject) {
            let id = if let Some(n) = caps.get(1) {
                n.as_str().to_string()
            } else if let Some(n) = caps.get(2) {
                n.as_str().to_string()
            } else {
                // If we are here then something went completly wrong.
                // to minimize the damage just return a `Subject::Simple`
                return Self::Simple(subject.to_string());
            };
            Self::PullRequest {
                id,
                description: subject.to_string(),
            }
        } else if subject.starts_with("fixup!") {
            Self::Fixup(subject.to_string())
        } else if let Some(caps) = UPDATE_REGEX.captures(subject) {
            let operation = SubtreeOperation::Update {
                subtree: caps[1].to_string(),
                git_ref: caps[2].to_string(),
            };
            Self::SubtreeCommit {
                operation,
                description: subject.to_string(),
            }
        } else if let Some(caps) = IMPORT_REGEX.captures(subject) {
            let operation = SubtreeOperation::Import {
                subtree: caps[1].to_string(),
                git_ref: caps[2].to_string(),
            };
            Self::SubtreeCommit {
                operation,
                description: subject.to_string(),
            }
        } else if let Some(caps) = SPLIT_REGEX.captures(subject) {
            let operation = SubtreeOperation::Split {
                subtree: caps[1].to_string(),
                git_ref: caps[2].to_string(),
            };
            Self::SubtreeCommit {
                operation,
                description: subject.to_string(),
            }
        } else if subject.to_lowercase().starts_with("remove ") {
            Self::Remove(subject.to_string())
        } else if subject.to_lowercase().starts_with("rename ") {
            Self::Rename(subject.to_string())
        } else if subject.to_lowercase().starts_with("revert ") {
            Self::Revert(subject.to_string())
        } else if let Some(caps) = PR_REGEX_BB.captures(subject) {
            let id = if let Some(n) = caps.get(1) {
                n.as_str().to_string()
            } else if let Some(n) = caps.get(2) {
                n.as_str().to_string()
            } else {
                // If we are here then something went completly wrong.
                // to minimize the damage just return a `Subject::Simple`
                return Self::Simple(subject.to_string());
            };
            Self::PullRequest {
                id,
                description: subject.to_string(),
            }
        } else if ADD_REGEX.is_match(subject) {
            Self::ConventionalCommit {
                breaking_change: false,
                category: Type::Feat,
                scope: None,
                description: subject.to_string(),
            }
        } else if FIX_REGEX.is_match(subject) {
            Self::ConventionalCommit {
                breaking_change: false,
                category: Type::Fix,
                scope: None,
                description: subject.to_string(),
            }
        } else if subject.to_lowercase().starts_with("deprecate ") {
            Self::ConventionalCommit {
                breaking_change: false,
                category: Type::Deprecate,
                scope: None,
                description: subject.to_string(),
            }
        } else if let Some(caps) = CONVENTIONAL_COMMIT_REGEX.captures(subject) {
            Self::parse_conventional_commit(&caps)
        } else {
            Self::Simple(subject.to_string())
        }
    }
}

impl Subject {
    #[must_use]
    pub fn icon(&self) -> &str {
        match self {
            Subject::Fixup(_) => "\u{f0e3} ",
            Subject::ConventionalCommit {
                breaking_change,
                category,
                ..
            } => {
                if *breaking_change {
                    "⚠ "
                } else {
                    match category {
                        Type::Archive => "\u{f53b} ",
                        Type::Build => "🔨",
                        Type::Change | Type::Improvement => "\u{e370} ",
                        Type::Chore => "\u{1F6A7} ", // unicode construction sign
                        Type::Ci => "\u{f085} ",
                        Type::Deprecate => "\u{f48e} ",
                        Type::Dev => "\u{1f6a9}",
                        Type::Deps => "\u{f487} ",
                        Type::Docs => "✎ ",
                        Type::Feat => "\u{1f381}", // unicode wrapped present
                        Type::Issue => " ",
                        Type::Fix => "\u{f188} ",
                        Type::I18n => "\u{fac9}",
                        Type::Other => "⁇ ",
                        Type::Perf => "\u{f9c4}",
                        Type::Refactor => "↺ ",
                        Type::Repo => " ",
                        Type::Security => " ",
                        Type::Style => "♥ ",
                        Type::Test => "\u{f45e} ",
                    }
                }
            }
            Subject::SubtreeCommit { operation, .. } => match operation {
                SubtreeOperation::Import { .. } => "⮈ ",
                SubtreeOperation::Split { .. } => "\u{f403} ",
                SubtreeOperation::Update { .. } => "\u{f419} ",
            },
            Subject::Simple(_) => "  ",
            Subject::Release { .. } => "\u{f412} ",
            Subject::Remove(_) => "\u{f48e} ",
            Subject::Rename(_) => "\u{f044} ",
            Subject::Revert(_) => " ",
            Subject::PullRequest { .. } => " ",
        }
    }
    fn parse_conventional_commit(caps: &Captures) -> Self {
        // eprintln!("{:#?}", caps);
        let mut cat_text = caps[1].to_string();
        let mut scope_text = caps
            .get(2)
            .map_or_else(|| "".to_string(), |_| caps[2].to_string());
        let mut rest_text = caps[3].to_string();
        let breaking_change = cat_text.ends_with('!')
            || scope_text.ends_with('!')
            || cat_text.to_lowercase().as_str() == "breaking change";
        if cat_text.ends_with('!') {
            cat_text.truncate(cat_text.len() - 1);
        }
        if scope_text.ends_with('!') {
            scope_text.truncate(scope_text.len() - 1);
        }

        if scope_text.len() >= 3 {
            scope_text = scope_text[1..scope_text.len() - 1].to_string();
        }

        let scope = if scope_text.is_empty() {
            None
        } else {
            Some(scope_text)
        };

        let category = match cat_text.to_lowercase().as_str() {
            "archive" => Type::Archive,
            "build" => Type::Build,
            "breaking change" | "change" => Type::Change,
            "chore" => Type::Chore,
            "ci" => Type::Ci,
            "deprecate" => Type::Deprecate,
            "deps" => Type::Deps,
            "dev" => Type::Dev,
            "docs" => Type::Docs,
            "add" | "feat" | "feature" => Type::Feat,
            "bugfix" | "fix" | "hotfix" => Type::Fix,
            "security" | "security fix" => Type::Security,
            "i18n" => Type::I18n,
            "gi" | "issue" => Type::Issue,
            "improvement" => Type::Improvement,
            "perf" => Type::Perf,
            "refactor" => Type::Refactor,
            "repo" => Type::Repo,
            "style" => Type::Style,
            "test" | "tests" => Type::Test,
            _ => Type::Other,
        };

        if category == Type::Other {
            rest_text = caps[0].to_string();
        }
        if breaking_change {
            let mut tmp = "! ".to_string();
            tmp.push_str(&rest_text);
            rest_text = tmp;
        }

        Self::ConventionalCommit {
            breaking_change,
            category,
            scope,
            description: rest_text,
        }
    }

    #[must_use]
    pub fn description(&self) -> &str {
        match self {
            Subject::ConventionalCommit { description, .. }
            | Subject::Fixup(description)
            | Subject::PullRequest { description, .. }
            | Subject::Release { description, .. }
            | Subject::SubtreeCommit { description, .. }
            | Subject::Remove(description)
            | Subject::Rename(description)
            | Subject::Revert(description)
            | Subject::Simple(description) => description,
        }
    }

    #[must_use]
    pub fn scope(&self) -> Option<String> {
        match self {
            Subject::ConventionalCommit { scope, .. } | Subject::Release { scope, .. } => {
                scope.clone()
            }
            Subject::SubtreeCommit { operation, .. } => match operation {
                SubtreeOperation::Import { subtree, .. }
                | SubtreeOperation::Split { subtree, .. }
                | SubtreeOperation::Update { subtree, .. } => Some(subtree.to_string()),
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Subject, SubtreeOperation, Type};

    #[test]
    fn archive() {
        let result = Subject::from("archive: windowmanager");
        let description = String::from("windowmanager");
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: false,
                category: Type::Archive,
                scope: None,
                description,
            },
        );
    }

    #[test]
    fn build() {
        let result = Subject::from("build(repo): Always use local file-expert");
        let description = String::from("Always use local file-expert");
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: false,
                category: Type::Build,
                scope: Some("repo".to_string()),
                description,
            },
        );
    }

    #[test]
    fn change() {
        let result = Subject::from("change!: Replace strncpy with memcpy");
        let description = "! Replace strncpy with memcpy".to_string();
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: true,
                category: Type::Change,
                scope: None,
                description,
            },
        );
        assert_eq!(result.icon(), "⚠ ");
        let result = Subject::from("change: Replace strncpy with memcpy");
        let description = "Replace strncpy with memcpy".to_string();
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: false,
                category: Type::Change,
                scope: None,
                description: description.clone(),
            },
        );
        assert_eq!(result.description(), description);
        assert_ne!(result.icon(), "⚠ ");

        let result = Subject::from("CHANGE Replace strncpy with memcpy");
        let description = "Replace strncpy with memcpy".to_string();
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: false,
                category: Type::Change,
                scope: None,
                description: description.clone(),
            },
        );
        assert_eq!(result.description(), description);
        assert_ne!(result.icon(), "⚠ ");
    }

    #[test]
    fn breaking_change() {
        let result = Subject::from("breaking change: Commits are now namedtupples");
        let description = "! Commits are now namedtupples".to_string();
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: true,
                category: Type::Change,
                scope: None,
                description: description.clone(),
            },
        );
        assert_eq!(result.description(), description);
        assert_eq!(result.icon(), "⚠ ");
    }

    #[test]
    fn ci() {
        let result = Subject::from("ci(srht): Fedora Rawhide run dist-rpm && qubes-builder");
        let description = "Fedora Rawhide run dist-rpm && qubes-builder".to_string();
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: false,
                category: Type::Ci,
                scope: Some("srht".to_string()),
                description,
            },
        );
    }
    #[test]
    fn deps() {
        let result = Subject::from("deps: Use thick Xlib bindings");
        let description = "Use thick Xlib bindings".to_string();
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: false,
                category: Type::Deps,
                scope: None,
                description,
            },
        );
    }
    #[test]
    fn docs() {
        let result = Subject::from("docs(readme): add xcb-util-xrm to dependencies' list");
        let description = "add xcb-util-xrm to dependencies' list".to_string();
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: false,
                category: Type::Docs,
                scope: Some("readme".to_string()),
                description,
            },
        );
    }

    #[test]
    fn scope_breaking_change() {
        let result = Subject::from("fix(search)!: This breaks the api");
        let description = "! This breaks the api".to_string();
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: true,
                category: Type::Fix,
                scope: Some("search".to_string()),
                description,
            },
        );
        assert_eq!(result.icon(), "⚠ ");
    }

    #[test]
    fn update_subtree() {
        let text = "Update :qubes-builder to 5e5301b8eac";
        let result = Subject::from(text);
        assert_eq!(
            result,
            Subject::SubtreeCommit {
                operation: SubtreeOperation::Update {
                    subtree: "qubes-builder".to_string(),
                    git_ref: "5e5301b8eac".to_string()
                },
                description: text.to_string()
            }
        );
    }

    #[test]
    fn split_subtree() {
        let text = "Split 'rust/' into commit 'baa77665cab9b8b25c7887e021280d8b55e2c9cb'";
        let result = Subject::from(text);
        assert_eq!(
            result,
            Subject::SubtreeCommit {
                operation: SubtreeOperation::Split {
                    subtree: "rust".to_string(),
                    git_ref: "baa77665cab9b8b25c7887e021280d8b55e2c9cb".to_string()
                },
                description: text.to_string()
            }
        );
    }

    #[test]
    fn import_subtree() {
        let text = ":php/composer-monorepo-plugin Import GH:github.com/beberlei/composer-monorepo-plugin⸪master";
        let result = Subject::from(text);
        assert_eq!(
            result,
            Subject::SubtreeCommit {
                operation: SubtreeOperation::Import {
                    subtree: "php/composer-monorepo-plugin".to_string(),
                    git_ref: "master".to_string()
                },
                description: text.to_string()
            }
        );
    }

    #[test]
    fn release1() {
        let text = "Release foo@v2.11.0";
        let result = Subject::from(text);
        assert_eq!(
            result,
            Subject::Release {
                version: "2.11.0".to_string(),
                scope: Some("foo".to_string()),
                description: text.to_string()
            }
        );
    }

    #[test]
    fn release2() {
        let text = "Release v2.11.0";
        let result = Subject::from(text);
        assert_eq!(
            result,
            Subject::Release {
                version: "2.11.0".to_string(),
                scope: None,
                description: text.to_string()
            }
        );

        let text = "Release 2.11.0";
        let result = Subject::from(text);
        assert_eq!(
            result,
            Subject::Release {
                version: "2.11.0".to_string(),
                scope: None,
                description: text.to_string()
            }
        );
    }

    #[test]
    fn revert() {
        let text = "Revert two commits breaking watching hotplug-status xenstore node";
        let result = Subject::from(text);
        assert_eq!(result, Subject::Revert(text.to_string()));
    }

    #[test]
    fn rename() {
        let text = "Rename ForkPointCalculation::Needed → InProgress";
        let result = Subject::from(text);
        assert_eq!(result, Subject::Rename(text.to_string()));
    }

    #[test]
    fn pr() {
        let text = "Merge remote-tracking branch 'origin/pr/126'";
        let result = Subject::from(text);
        assert_eq!(
            result,
            Subject::PullRequest {
                id: "126".to_string(),
                description: text.to_string()
            }
        );
    }

    #[test]
    fn pr_bitbucket() {
        let text = "Merge pull request #7771 in FOO/bar from feature/asdqwert to development";
        let result = Subject::from(text);
        assert_eq!(
            result,
            Subject::PullRequest {
                id: "7771".to_string(),
                description: text.to_string()
            }
        );
    }

    #[test]
    fn security() {
        let text = "security: Fix CSV-FOO-1234";
        let result = Subject::from(text);
        let description = "Fix CSV-FOO-1234".to_string();
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: false,
                category: Type::Security,
                scope: None,
                description
            }
        );

        let text = "security fix: Fix CSV-FOO-1234";
        let result = Subject::from(text);
        let description = "Fix CSV-FOO-1234".to_string();
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: false,
                category: Type::Security,
                scope: None,
                description
            }
        );
    }

    #[test]
    fn other() {
        let text = "Makefile: replace '-' in plugins_var";
        let result = Subject::from(text);
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: false,
                category: Type::Other,
                scope: None,
                description: "Makefile: replace '-' in plugins_var".to_string()
            }
        );
    }

    #[test]
    fn deprecate() {
        let text = "deprecate: Mark Foo() as deprecated";
        let result = Subject::from(text);
        let description = "Mark Foo() as deprecated".to_string();
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: false,
                category: Type::Deprecate,
                scope: None,
                description
            }
        );

        let text = "Deprecate Foo() use Bar() instead";
        let result = Subject::from(text);
        let description = "Deprecate Foo() use Bar() instead".to_string();
        assert_eq!(
            result,
            Subject::ConventionalCommit {
                breaking_change: false,
                category: Type::Deprecate,
                scope: None,
                description
            }
        );
    }
}
