//! The backlog in docs/backlog/, parsed into typed items.
//!
//! The format is specified in docs/backlog/README.md#item-format: each epic file holds
//! features (`##` headings) and their tasks (`###` headings). Each item has a metadata
//! line, and each task has a `Done when:` line and optional trailers. Parsing collects
//! every format error with its file and line rather than stopping at the first one.
//!
//! The commands load the repository backlog through [`with_repository_backlog`], which
//! refuses to run on a backlog with format errors.
//!
//! This ports the parser of docs/backlog/tools/backlog.py (FND-F07). Until that script
//! is deleted, error messages match its output exactly: the goldens in
//! `xtask/tests/fixtures/backlog/` were produced by it. An item with an unknown status,
//! priority, milestone or type is reported and then left out, where the script kept it
//! as text; only already-broken backlogs can see the difference.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::Exit;
use crate::output;

/// A task's or feature's status, in the README's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    ToDo,
    InProgress,
    InReview,
    Blocked,
    Done,
    WontDo,
}

impl Status {
    const ALL: [(Status, &'static str); 6] = [
        (Status::ToDo, "To Do"),
        (Status::InProgress, "In Progress"),
        (Status::InReview, "In Review"),
        (Status::Blocked, "Blocked"),
        (Status::Done, "Done"),
        (Status::WontDo, "Won't Do"),
    ];

    fn parse(text: &str) -> Option<Status> {
        Self::ALL
            .iter()
            .find(|(_, name)| *name == text)
            .map(|(status, _)| *status)
    }

    /// Done and Won't Do both satisfy a dependency.
    pub(crate) fn is_closed(self) -> bool {
        matches!(self, Status::Done | Status::WontDo)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = Self::ALL
            .iter()
            .find(|(status, _)| status == self)
            .map_or("", |(_, name)| *name);
        f.write_str(name)
    }
}

/// `P0` (critical path) to `P3` (nice to have). Ordered most urgent first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Priority {
    P0,
    P1,
    P2,
    P3,
}

impl Priority {
    const ALL: [Priority; 4] = [Priority::P0, Priority::P1, Priority::P2, Priority::P3];

    fn parse(text: &str) -> Option<Priority> {
        Self::ALL
            .into_iter()
            .find(|priority| priority.to_string() == text)
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let number = match self {
            Priority::P0 => 0,
            Priority::P1 => 1,
            Priority::P2 => 2,
            Priority::P3 => 3,
        };
        write!(f, "P{number}")
    }
}

/// A milestone, `M0` to `M18` (docs/backlog/README.md#milestones).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Milestone(u8);

impl Milestone {
    const LAST: u8 = 18;

    /// Every milestone, in order.
    pub(crate) fn all() -> impl Iterator<Item = Milestone> {
        (0..=Self::LAST).map(Milestone)
    }

    fn parse(text: &str) -> Option<Milestone> {
        Self::all().find(|milestone| milestone.to_string() == text)
    }
}

impl fmt::Display for Milestone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "M{}", self.0)
    }
}

/// The type field of the metadata line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemType {
    Feature,
    Task,
    Spike,
    Story,
    Bug,
}

impl ItemType {
    const ALL: [(ItemType, &'static str); 5] = [
        (ItemType::Feature, "Feature"),
        (ItemType::Task, "Task"),
        (ItemType::Spike, "Spike"),
        (ItemType::Story, "Story"),
        (ItemType::Bug, "Bug"),
    ];

    fn parse(text: &str) -> Option<ItemType> {
        Self::ALL
            .iter()
            .find(|(_, name)| *name == text)
            .map(|(item_type, _)| *item_type)
    }
}

impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = Self::ALL
            .iter()
            .find(|(item_type, _)| item_type == self)
            .map_or("", |(_, name)| *name);
        f.write_str(name)
    }
}

/// Whether an item is a feature (`##`) or a task (`###`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Level {
    Feature,
    Task,
}

/// One feature or task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Item {
    pub(crate) id: String,
    pub(crate) level: Level,
    pub(crate) status: Status,
    pub(crate) title: String,
    pub(crate) priority: Priority,
    pub(crate) milestone: Milestone,
    /// The type field: `Feature`, `Task`, `Spike`, `Story` or `Bug`.
    pub(crate) kind: ItemType,
    /// Item ids from `deps:`; empty for `none`.
    pub(crate) deps: Vec<String>,
    pub(crate) labels: Vec<String>,
    /// The `ref:` field, or empty.
    pub(crate) reference: String,
    /// The epic file's name, for example `02-fnd-foundation.md`.
    pub(crate) file: String,
    /// The 1-based line of the heading.
    pub(crate) line: usize,
    pub(crate) epic: String,
    /// For a task, the id of the feature it sits under.
    pub(crate) feature: Option<String>,
    /// For a task, the text after `Done when: `.
    pub(crate) done_when: Option<String>,
    /// Trailer lines (`Owner:`, `PR:`, `Blocked:`, `Won't do:`, `Notes:`) in file
    /// order, as key (without the colon) and value.
    pub(crate) trailers: Vec<(String, String)>,
    /// For a feature, the ids of its tasks in file order.
    pub(crate) tasks: Vec<String>,
}

/// Every item, in file order, with the epic build order from the README.
#[derive(Debug, Clone, Default)]
pub(crate) struct Backlog {
    pub(crate) items: Vec<Item>,
    index: BTreeMap<String, usize>,
    /// Epic codes in the order of the README epic table.
    pub(crate) epic_order: Vec<String>,
}

impl Backlog {
    pub(crate) fn get(&self, id: &str) -> Option<&Item> {
        self.index.get(id).and_then(|index| self.items.get(*index))
    }

    pub(crate) fn tasks(&self) -> impl Iterator<Item = &Item> {
        self.items.iter().filter(|item| item.level == Level::Task)
    }

    /// The tasks `item` waits for: its own task dependencies, and for each feature it
    /// (or, for a task, its feature) depends on, that feature's tasks at or before
    /// `item`'s milestone (docs/backlog/README.md#dependencies). Unknown ids are skipped.
    pub(crate) fn effective_deps(&self, item: &Item) -> BTreeSet<String> {
        let inherited = match (item.level, &item.feature) {
            (Level::Task, Some(feature)) => self.get(feature).map(|feature| &feature.deps[..]),
            _ => None,
        };
        let mut waits_for = BTreeSet::new();
        for dep in item.deps.iter().chain(inherited.into_iter().flatten()) {
            let Some(dep) = self.get(dep) else { continue };
            match dep.level {
                Level::Task => {
                    waits_for.insert(dep.id.clone());
                }
                Level::Feature => waits_for.extend(
                    dep.tasks
                        .iter()
                        .filter_map(|task| self.get(task))
                        .filter(|task| task.milestone <= item.milestone && task.id != item.id)
                        .map(|task| task.id.clone()),
                ),
            }
        }
        waits_for
    }

    /// Adds `item`, registering a task with its feature. A duplicate id replaces the
    /// earlier item in place, as backlog.py does.
    fn insert(&mut self, item: Item) {
        let feature = item
            .feature
            .as_deref()
            .and_then(|feature| self.index.get(feature).copied());
        if let Some(feature) = feature.and_then(|index| self.items.get_mut(index)) {
            feature.tasks.push(item.id.clone());
        }
        if let Some(slot) = self
            .index
            .get(&item.id)
            .and_then(|index| self.items.get_mut(*index))
        {
            *slot = item;
        } else {
            self.index.insert(item.id.clone(), self.items.len());
            self.items.push(item);
        }
    }
}

/// A parsed backlog and every format error found, as `file:line: message`.
#[derive(Debug, Clone, Default)]
pub(crate) struct Parsed {
    pub(crate) backlog: Backlog,
    pub(crate) errors: Vec<String>,
}

/// The repository's backlog directory.
pub(crate) fn repository_dir() -> PathBuf {
    crate::repo_root().join("docs/backlog")
}

/// At most this many format errors are printed before a command refuses to run.
const ERRORS_SHOWN: usize = 20;

/// Loads the repository backlog and runs `command` on it. With format errors, prints
/// the first few and fails instead: commands that select or count work need a valid
/// backlog.
pub(crate) fn with_repository_backlog<W: Write>(
    out: &mut W,
    command: impl FnOnce(&Backlog, &mut W) -> Exit,
) -> Exit {
    let Parsed { backlog, errors } = match load(&repository_dir()) {
        Ok(parsed) => parsed,
        Err(error) => {
            output::line(out, &format!("error: {error}"));
            return Exit::CheckFailed;
        }
    };
    if errors.is_empty() {
        return command(&backlog, out);
    }
    for error in errors.iter().take(ERRORS_SHOWN) {
        output::line(out, &format!("error: {error}"));
    }
    output::line(
        out,
        "backlog has format errors; run `python3 docs/backlog/tools/backlog.py check`",
    );
    Exit::CheckFailed
}

/// Reads and parses the backlog in `dir`: its README.md and every other `.md` file.
pub(crate) fn load(dir: &Path) -> Result<Parsed, String> {
    let read = |name: &str| {
        std::fs::read_to_string(dir.join(name))
            .map_err(|error| format!("could not read {}: {error}", dir.join(name).display()))
    };
    let readme = read("README.md")?;
    let mut names = Vec::new();
    let entries = std::fs::read_dir(dir)
        .map_err(|error| format!("could not read {}: {error}", dir.display()))?;
    for entry in entries {
        let name = entry
            .map_err(|error| format!("could not read {}: {error}", dir.display()))?
            .file_name()
            .to_string_lossy()
            .into_owned();
        if Path::new(&name)
            .extension()
            .is_some_and(|extension| extension == "md")
            && name != "README.md"
        {
            names.push(name);
        }
    }
    names.sort();
    let mut files = Vec::with_capacity(names.len());
    for name in names {
        let text = read(&name)?;
        files.push((name, text));
    }
    Ok(parse(&readme, &files))
}

/// Parses the README (for the epic order) and the epic files, given as
/// `(file name, contents)` in name order.
pub(crate) fn parse(readme: &str, files: &[(String, String)]) -> Parsed {
    let mut parsed = Parsed {
        backlog: Backlog {
            epic_order: epic_order(readme),
            ..Backlog::default()
        },
        errors: Vec::new(),
    };
    for (name, text) in files {
        parse_file(name, text, &mut parsed);
    }
    parsed
}

/// Epic codes from the README epic table rows (`| 2 | FND | [Title](02-….md) | … |`).
fn epic_order(readme: &str) -> Vec<String> {
    readme.lines().filter_map(epic_row_code).collect()
}

fn epic_row_code(line: &str) -> Option<String> {
    let rest = line.strip_prefix("| ")?;
    let (number, rest) = rest.split_once(" | ")?;
    let (code, rest) = rest.split_once(" | [")?;
    let (_, after_title) = rest.split_once("](")?;
    let (target, _) = after_title.split_once(')')?;
    let valid = !number.is_empty()
        && number.bytes().all(|byte| byte.is_ascii_digit())
        && is_epic_code(code)
        && !target.is_empty();
    valid.then(|| code.to_owned())
}

/// `[A-Z][A-Z-]*`.
fn is_epic_code(text: &str) -> bool {
    text.bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_uppercase())
        && text
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte == b'-')
}

const TRAILER_KEYS: [&str; 5] = ["Owner", "PR", "Blocked", "Won't do", "Notes"];

fn parse_file(name: &str, text: &str, parsed: &mut Parsed) {
    let lines: Vec<&str> = text.split('\n').collect();
    let code = lines
        .first()
        .and_then(|first| first.strip_prefix("# "))
        .and_then(|rest| rest.split_once(": "))
        .map(|(code, _)| code)
        .filter(|code| is_epic_code(code));
    let Some(code) = code else {
        parsed
            .errors
            .push(format!("{name}:1: first line must be '# <EPIC>: <title>'"));
        return;
    };
    if !is_epic_file_name(name, code) {
        parsed.errors.push(format!(
            "{name}: file name must be '<NN>-{}-<slug>.md'",
            code.to_lowercase()
        ));
    }
    let file = EpicFile {
        name,
        code,
        lines: &lines,
    };
    let mut feature = None;
    let mut index = 0;
    while let Some(line) = lines.get(index) {
        index = if line.starts_with("## ") || line.starts_with("### ") {
            parse_item(&file, index, &mut feature, parsed)
        } else {
            index + 1
        };
    }
}

/// One epic file being parsed.
struct EpicFile<'a> {
    name: &'a str,
    code: &'a str,
    lines: &'a [&'a str],
}

impl EpicFile<'_> {
    fn line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).copied()
    }
}

/// Parses the item whose heading is at `index`, recording errors and inserting the item
/// if its fields are valid. Returns the index of the next line to read. `feature` is the
/// id of the feature that later tasks belong to.
fn parse_item(
    file: &EpicFile<'_>,
    index: usize,
    feature: &mut Option<String>,
    parsed: &mut Parsed,
) -> usize {
    let location = format!("{}:{}", file.name, index + 1);
    let line = file.line(index).unwrap_or_default();
    let Some(heading) = Heading::parse(line) else {
        parsed
            .errors
            .push(format!("{location}: malformed heading: {line}"));
        return index + 1;
    };
    let meta_line = file.line(index + 1).unwrap_or_default();
    let Some(meta) = Metadata::parse(meta_line) else {
        parsed.errors.push(format!(
            "{location}: malformed metadata line after {}: {}",
            heading.id,
            python_repr(meta_line)
        ));
        return index + 1;
    };
    let id = heading.id;
    let fields = Fields::check(&heading, &meta, &location, &mut parsed.errors);
    let kind = ItemType::parse(meta.kind);
    let (level, body) = if heading.is_feature {
        check_feature(file, index, id, kind, &location, &mut parsed.errors);
        *feature = Some(id.to_owned());
        (Level::Feature, Body::empty(index + 2))
    } else {
        let body = parse_task(file, index, id, kind, &location, &mut parsed.errors);
        if feature.is_none() {
            parsed
                .errors
                .push(format!("{location}: task {id} appears before any feature"));
        }
        let has = |key: &str| body.trailers.iter().any(|(existing, _)| existing == key);
        if fields.status == Some(Status::Blocked) && !has("Blocked") {
            parsed.errors.push(format!(
                "{location}: {id}: Blocked tasks need a 'Blocked:' line"
            ));
        }
        if fields.status == Some(Status::WontDo) && !has("Won't do") {
            parsed.errors.push(format!(
                "{location}: {id}: Won't Do tasks need a \"Won't do:\" line"
            ));
        }
        (Level::Task, body)
    };
    if let Some(existing) = parsed.backlog.get(id) {
        parsed.errors.push(format!(
            "{location}: duplicate id {id} (also {}:{})",
            existing.file, existing.line
        ));
    }
    let next = body.next;
    if let (Some(status), Some(priority), Some(milestone), Some(kind)) =
        (fields.status, fields.priority, fields.milestone, kind)
    {
        let item = Item {
            id: id.to_owned(),
            level,
            status,
            title: heading.title.to_owned(),
            priority,
            milestone,
            kind,
            deps: fields.deps,
            labels: split_labels(meta.labels),
            reference: meta.reference.trim().to_owned(),
            file: file.name.to_owned(),
            line: index + 1,
            epic: file.code.to_owned(),
            feature: if level == Level::Task {
                feature.clone()
            } else {
                None
            },
            done_when: body.done_when,
            trailers: body.trailers,
            tasks: Vec::new(),
        };
        parsed.backlog.insert(item);
    }
    next
}

/// The fields every item has, checked; `None` where the text is not a known value.
struct Fields {
    status: Option<Status>,
    priority: Option<Priority>,
    milestone: Option<Milestone>,
    deps: Vec<String>,
}

impl Fields {
    fn check(
        heading: &Heading<'_>,
        meta: &Metadata<'_>,
        location: &str,
        errors: &mut Vec<String>,
    ) -> Fields {
        let id = heading.id;
        let status = Status::parse(heading.status);
        if status.is_none() {
            errors.push(format!(
                "{location}: {id}: unknown status '{}'",
                heading.status
            ));
        }
        let priority = Priority::parse(meta.priority);
        if priority.is_none() {
            errors.push(format!(
                "{location}: {id}: unknown priority '{}'",
                meta.priority
            ));
        }
        let milestone = Milestone::parse(meta.milestone);
        if milestone.is_none() {
            errors.push(format!(
                "{location}: {id}: unknown milestone '{}'",
                meta.milestone
            ));
        }
        let deps: Vec<String> = if meta.deps.trim() == "none" {
            Vec::new()
        } else {
            meta.deps
                .split(',')
                .map(|dep| dep.trim().to_owned())
                .collect()
        };
        if deps.iter().any(String::is_empty) {
            errors.push(format!("{location}: {id}: empty dependency entry"));
        }
        Fields {
            status,
            priority,
            milestone,
            deps,
        }
    }
}

/// What follows a task's metadata line, and the index of the line after it.
struct Body {
    next: usize,
    done_when: Option<String>,
    trailers: Vec<(String, String)>,
}

impl Body {
    fn empty(next: usize) -> Body {
        Body {
            next,
            done_when: None,
            trailers: Vec::new(),
        }
    }
}

fn check_feature(
    file: &EpicFile<'_>,
    index: usize,
    id: &str,
    kind: Option<ItemType>,
    location: &str,
    errors: &mut Vec<String>,
) {
    let code = file.code;
    if !is_item_id(id, code, "F", 2..=3) {
        errors.push(format!("{location}: feature id '{id}' must be {code}-Fnn"));
    }
    if kind != Some(ItemType::Feature) {
        errors.push(format!(
            "{location}: {id}: '##' items must have type Feature"
        ));
    }
    if !file
        .line(index + 2)
        .is_some_and(|line| line.starts_with("Scope: "))
    {
        errors.push(format!("{location}: {id}: missing 'Scope:' line"));
    }
}

/// Checks a task's id and type, and reads its `Done when:` line and trailers.
fn parse_task(
    file: &EpicFile<'_>,
    index: usize,
    id: &str,
    kind: Option<ItemType>,
    location: &str,
    errors: &mut Vec<String>,
) -> Body {
    let code = file.code;
    if !is_item_id(id, code, "", 4..=4) {
        errors.push(format!("{location}: task id '{id}' must be {code}-nnnn"));
    }
    if matches!(kind, None | Some(ItemType::Feature)) {
        errors.push(format!(
            "{location}: {id}: task type must be one of ['Bug', 'Spike', 'Story', 'Task']"
        ));
    }
    let mut body = Body::empty(index + 2);
    let done_when = file.line(body.next).and_then(|line| {
        line.strip_prefix("Done when: ")
            .filter(|_| line.chars().count() > 20)
    });
    if let Some(text) = done_when {
        body.done_when = Some(text.to_owned());
        body.next += 1;
    } else {
        errors.push(format!(
            "{location}: {id}: missing or empty 'Done when:' line"
        ));
    }
    while let Some((key, value)) = file.line(body.next).and_then(trailer) {
        if let Some(slot) = body
            .trailers
            .iter_mut()
            .find(|(existing, _)| *existing == key)
        {
            slot.1 = value;
        } else {
            body.trailers.push((key, value));
        }
        body.next += 1;
    }
    body
}

fn split_labels(labels: &str) -> Vec<String> {
    let labels = labels.trim();
    if labels.is_empty() {
        Vec::new()
    } else {
        labels
            .split(',')
            .map(|label| label.trim().to_owned())
            .collect()
    }
}

fn trailer(line: &str) -> Option<(String, String)> {
    TRAILER_KEYS.iter().find_map(|key| {
        line.strip_prefix(key)
            .and_then(|rest| rest.strip_prefix(':'))
            .map(|value| ((*key).to_owned(), value.trim().to_owned()))
    })
}

/// `<NN>-<code in lowercase>-<slug>.md`, the slug being `[a-z0-9-]+`.
fn is_epic_file_name(name: &str, code: &str) -> bool {
    let prefix = format!("-{}-", code.to_lowercase());
    let Some(slug) = name
        .get(2..)
        .and_then(|rest| rest.strip_prefix(prefix.as_str()))
        .and_then(|rest| rest.strip_suffix(".md"))
    else {
        return false;
    };
    name.bytes().take(2).all(|byte| byte.is_ascii_digit())
        && !slug.is_empty()
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

/// `<code>-<marker><digits>`, with a digit count in `digits`.
fn is_item_id(id: &str, code: &str, marker: &str, digits: std::ops::RangeInclusive<usize>) -> bool {
    id.strip_prefix(code)
        .and_then(|rest| rest.strip_prefix('-'))
        .and_then(|rest| rest.strip_prefix(marker))
        .is_some_and(|number| {
            digits.contains(&number.len()) && number.bytes().all(|byte| byte.is_ascii_digit())
        })
}

/// A `##`/`###` heading: `[<status>] <id> <title>`.
#[derive(Debug)]
struct Heading<'a> {
    is_feature: bool,
    status: &'a str,
    id: &'a str,
    title: &'a str,
}

impl<'a> Heading<'a> {
    fn parse(line: &'a str) -> Option<Self> {
        let (is_feature, rest) = match line.strip_prefix("### ") {
            Some(rest) => (false, rest),
            None => (true, line.strip_prefix("## ")?),
        };
        let (status, rest) = rest.strip_prefix('[')?.split_once("] ")?;
        let (id, title) = rest.split_once(' ')?;
        let valid = !status.is_empty()
            && !id.is_empty()
            && !id.contains(char::is_whitespace)
            && !title.is_empty();
        valid.then_some(Heading {
            is_feature,
            status,
            id,
            title,
        })
    }
}

/// The metadata line: `P0 · M2 · Task · deps: … [· labels: …] [· ref: …]`.
#[derive(Debug)]
struct Metadata<'a> {
    priority: &'a str,
    milestone: &'a str,
    kind: &'a str,
    deps: &'a str,
    labels: &'a str,
    reference: &'a str,
}

impl<'a> Metadata<'a> {
    const SEPARATOR: &'static str = " · ";

    fn parse(line: &'a str) -> Option<Self> {
        let (priority, rest) = line.split_once(Self::SEPARATOR)?;
        let (milestone, rest) = rest.split_once(Self::SEPARATOR)?;
        let (kind, rest) = rest.split_once(Self::SEPARATOR)?;
        let well_formed = priority.len() == 2
            && priority.starts_with('P')
            && priority.as_bytes().get(1).is_some_and(u8::is_ascii_digit)
            && milestone.len() > 1
            && milestone.starts_with('M')
            && milestone.bytes().skip(1).all(|byte| byte.is_ascii_digit())
            && !kind.is_empty()
            && kind.chars().all(|c| c.is_alphanumeric() || c == '_');
        if !well_formed {
            return None;
        }
        let rest = rest.strip_prefix("deps: ")?;
        // A field value never contains `·`, except `ref:`, which runs to the end.
        let (deps, rest) = split_field(rest)?;
        let (labels, rest) = match rest.and_then(|rest| rest.strip_prefix("labels: ")) {
            Some(labels) => split_field(labels)?,
            None => ("", rest),
        };
        let reference = match rest {
            None => "",
            Some(rest) => rest
                .strip_prefix("ref: ")
                .filter(|value| !value.is_empty())?,
        };
        Some(Metadata {
            priority,
            milestone,
            kind,
            deps,
            labels,
            reference,
        })
    }
}

/// Splits a non-empty field value without `·` from what follows its separator.
fn split_field(text: &str) -> Option<(&str, Option<&str>)> {
    let (value, rest) = match text.find('·') {
        None => (text, None),
        Some(position) => {
            let value = text.get(..position)?.strip_suffix(' ')?;
            let rest = text.get(position..)?.strip_prefix("· ")?;
            (value, Some(rest))
        }
    };
    (!value.is_empty()).then_some((value, rest))
}

/// Python's `repr()` of a string, which backlog.py uses in error messages.
fn python_repr(text: &str) -> String {
    let quote = if text.contains('\'') && !text.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut repr = String::with_capacity(text.len() + 2);
    repr.push(quote);
    for c in text.chars() {
        match c {
            '\\' => repr.push_str("\\\\"),
            '\n' => repr.push_str("\\n"),
            '\r' => repr.push_str("\\r"),
            '\t' => repr.push_str("\\t"),
            c if c == quote => {
                repr.push('\\');
                repr.push(c);
            }
            c if c.is_control() => {
                // Control characters are all below U+0100, so two hex digits suffice.
                let code = u32::from(c);
                repr.push_str("\\x");
                for shift in [4, 0] {
                    repr.push(char::from_digit((code >> shift) & 0xf, 16).unwrap_or('0'));
                }
            }
            c => repr.push(c),
        }
    }
    repr.push(quote);
    repr
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::{Value, json};

    use super::{Item, Level, load, parse, python_repr};

    fn fixture(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/backlog")
            .join(name)
    }

    fn golden(name: &str) -> String {
        std::fs::read_to_string(fixture(name)).unwrap_or_else(|error| panic!("{name}: {error}"))
    }

    fn to_json(item: &Item) -> Value {
        let trailers: serde_json::Map<String, Value> = item
            .trailers
            .iter()
            .map(|(key, value)| (key.clone(), Value::from(value.clone())))
            .collect();
        json!({
            "id": item.id,
            "level": match item.level { Level::Feature => "feature", Level::Task => "task" },
            "status": item.status.to_string(),
            "title": item.title,
            "priority": item.priority.to_string(),
            "milestone": item.milestone.to_string(),
            "type": item.kind.to_string(),
            "deps": item.deps,
            "labels": item.labels,
            "ref": item.reference,
            "file": item.file,
            "line": item.line,
            "epic": item.epic,
            "feature": item.feature,
            "done_when": item.done_when,
            "trailers": trailers,
            "tasks": item.tasks,
        })
    }

    #[test]
    fn the_fixture_parses_to_the_same_items_as_backlog_py() {
        let parsed = load(&fixture("valid")).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(parsed.errors, Vec::<String>::new());
        let expected: Value = serde_json::from_str(&golden("valid.items.json"))
            .unwrap_or_else(|error| panic!("{error}"));
        let actual: Vec<Value> = parsed.backlog.items.iter().map(to_json).collect();
        assert_eq!(Value::from(actual), expected);
        assert_eq!(parsed.backlog.epic_order, ["AAA", "BBB"]);
    }

    #[test]
    fn format_errors_match_backlog_py_with_file_and_line() {
        let parsed = load(&fixture("malformed")).unwrap_or_else(|error| panic!("{error}"));
        let expected: Vec<String> = golden("malformed.errors.txt")
            .lines()
            .map(str::to_owned)
            .collect();
        assert_eq!(parsed.errors, expected);
    }

    #[test]
    fn the_readme_example_parses() {
        let readme = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/backlog/README.md"),
        )
        .unwrap_or_else(|error| panic!("{error}"));
        let example = readme
            .split_once("```markdown\n")
            .and_then(|(_, rest)| rest.split_once("```"))
            .map_or_else(
                || panic!("README.md has no markdown example"),
                |(example, _)| example,
            );
        let file = (
            "07-stor-storage.md".to_owned(),
            format!("# STOR: Storage\n\n{example}"),
        );
        let parsed = parse("", &[file]);
        assert_eq!(parsed.errors, Vec::<String>::new());
        let task = parsed
            .backlog
            .get("STOR-0031")
            .unwrap_or_else(|| panic!("no task"));
        assert_eq!(task.deps, ["STOR-0030"]);
        assert_eq!(task.labels, ["storage", "decoder"]);
        assert_eq!(task.reference, "storage/…/OffsetIndex.java lookup @4.4.0");
        assert_eq!(task.feature.as_deref(), Some("STOR-F04"));
        let feature = parsed
            .backlog
            .get("STOR-F04")
            .unwrap_or_else(|| panic!("no feature"));
        assert_eq!(feature.tasks, ["STOR-0031"]);
    }

    #[test]
    fn python_repr_picks_quotes_and_escapes_like_python() {
        assert_eq!(python_repr(""), "''");
        assert_eq!(python_repr("P0 - M0"), "'P0 - M0'");
        assert_eq!(python_repr("won't"), "\"won't\"");
        assert_eq!(python_repr("it's \"x\""), "'it\\'s \"x\"'");
        assert_eq!(python_repr("a\\b\tc\u{1}·"), "'a\\\\b\\tc\\x01·'");
    }
}
