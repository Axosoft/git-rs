use error::protocol::{Error, ProcessError::Parsing};
use nom::{digit1, oct_digit1};
use util::parse::{sha, parse_u32};

#[derive(Clone, Debug, Serialize)]
pub enum Status {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Unmerged,
}

named!(parse_status<&str, Option<Status>>,
    switch!(take!(1),
        "M" => value!(Some(Status::Modified)) |
        "A" => value!(Some(Status::Added)) |
        "D" => value!(Some(Status::Deleted)) |
        "R" => value!(Some(Status::Renamed)) |
        "C" => value!(Some(Status::Copied)) |
        "?" => value!(Some(Status::Untracked)) |
        "." => value!(None)
    )
);

named!(parse_status_unmerged<&str, Status>,
    switch!(take!(1),
        "A" => value!(Status::Added) |
        "D" => value!(Status::Deleted) |
        "U" => value!(Status::Unmerged)
    )
);

#[derive(Debug)]
pub struct FileModeStatus {
    head: u32,
    index: u32,
    worktree: u32,
}

named!(parse_file_mode<&str, FileModeStatus>,
    do_parse!(
        head: oct_digit1 >>
        char!(' ') >>
        index: oct_digit1 >>
        char!(' ') >>
        worktree: oct_digit1 >>
        (FileModeStatus {
            head: parse_u32(head, 8),
            index: parse_u32(index, 8),
            worktree: parse_u32(worktree, 8)
        })
    )
);

#[derive(Debug)]
pub struct UnmergedFileModeStatus {
    stage_1: u32,
    stage_2: u32,
    stage_3: u32,
    worktree: u32,
}

named!(parse_unmerged_file_mode<&str, UnmergedFileModeStatus>,
    do_parse!(
        stage_1: oct_digit1 >>
        char!(' ') >>
        stage_2: oct_digit1 >>
        char!(' ') >>
        stage_3: oct_digit1 >>
        char!(' ') >>
        worktree: oct_digit1 >>
        (UnmergedFileModeStatus {
            stage_1: parse_u32(stage_1, 8),
            stage_2: parse_u32(stage_2, 8),
            stage_3: parse_u32(stage_3, 8),
            worktree: parse_u32(worktree, 8)
        })
    )
);

#[derive(Clone, Debug, Serialize)]
pub enum ScoreType {
    Renamed,
    Copied,
}

#[derive(Clone, Debug, Serialize)]
pub struct Score {
    score_type: ScoreType,
    percentage: u32,
}

named!(parse_score<&str, Score>,
    do_parse!(
        score_type: switch!(take!(1), "R" => value!(ScoreType::Renamed) | "C" => value!(ScoreType::Copied)) >>
        percentage: digit1 >>
        (Score { score_type, percentage: parse_u32(percentage, 10) })
    )
);

#[derive(Clone, Debug, Serialize)]
pub struct SubmoduleStatus {
    commit_changed: bool,
    has_tracked_changes: bool,
    has_untracked_changes: bool,
}

named!(parse_submodule_status<&str, Option<SubmoduleStatus>>,
    do_parse!(
        commit_changed: switch!(take!(1), "C" => value!(true) | "." => value!(false)) >>
        has_tracked_changes: switch!(take!(1), "M" => value!(true) | "." => value!(false)) >>
        has_untracked_changes: switch!(take!(1), "U" => value!(true) | "." => value!(false)) >>
        (Some(SubmoduleStatus { commit_changed, has_tracked_changes, has_untracked_changes }))
    )
);

named!(parse_maybe_submodule_status<&str, Option<SubmoduleStatus>>,
    switch!(take!(1),
        "S" => call!(parse_submodule_status) |
        "N" => do_parse!(take!(3) >> (None))
    )
);

#[derive(Clone, Debug, Serialize)]
pub struct StatusOids {
    head: String,
    index: String,
}

named!(parse_status_oids<&str, StatusOids>,
    do_parse!(
        head: sha >>
        char!(' ') >>
        index: sha >>
        (StatusOids { head: head.to_string(), index: index.to_string() })
    )
);

#[derive(Debug)]
pub struct UnmergedStatusOids {
    stage_1: String,
    stage_2: String,
    stage_3: String,
}

named!(parse_unmerged_status_oids<&str, UnmergedStatusOids>,
    do_parse!(
        stage_1: sha >>
        char!(' ') >>
        stage_2: sha >>
        char!(' ') >>
        stage_3: sha >>
        (UnmergedStatusOids {
            stage_1: stage_1.to_string(),
            stage_2: stage_2.to_string(),
            stage_3: stage_3.to_string()
        })
    )
);

#[derive(Debug)]
pub enum StatusEntry {
    OrdinaryStatusEntry(OrdinaryStatusEntry),
    CopiedOrRenamedStatusEntry(CopiedOrRenamedStatusEntry),
    UnmergedStatusEntry(UnmergedStatusEntry),
    UntrackedStatusEntry(UntrackedStatusEntry),
    IgnoredStatusEntry(IgnoredStatusEntry),
}

#[derive(Debug)]
pub struct OrdinaryStatusEntry {
    staged_status: Option<Status>,
    unstaged_status: Option<Status>,
    submodule_status: Option<SubmoduleStatus>,
    file_mode: FileModeStatus,
    oids: StatusOids,
    path: String,
}

#[derive(Debug)]
pub struct CopiedOrRenamedStatusEntry {
    staged_status: Option<Status>,
    unstaged_status: Option<Status>,
    submodule_status: Option<SubmoduleStatus>,
    file_mode: FileModeStatus,
    oids: StatusOids,
    score: Score,
    path: String,
    original_path: String,
}

#[derive(Debug)]
pub struct UnmergedStatusEntry {
    staged_status: Status,
    unstaged_status: Status,
    submodule_status: Option<SubmoduleStatus>,
    file_mode: UnmergedFileModeStatus,
    oids: UnmergedStatusOids,
    path: String,
}

#[derive(Debug)]
pub struct UntrackedStatusEntry {
    path: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct IgnoredStatusEntry {
    path: String,
}

named!(parse_ordinary_status_entry<&str, StatusEntry>,
    do_parse!(
        staged_status: parse_status >>
        unstaged_status: parse_status >>
        char!(' ') >>
        submodule_status: parse_maybe_submodule_status >>
        char!(' ') >>
        file_mode: parse_file_mode >>
        char!(' ') >>
        oids: parse_status_oids >>
        char!(' ') >>
        path: take_until!("\n") >>
        (StatusEntry::OrdinaryStatusEntry(OrdinaryStatusEntry {
            staged_status,
            unstaged_status,
            submodule_status,
            file_mode,
            oids,
            path: path.to_string(),
        }))
    )
);

named!(parse_copied_or_renamed_status_entry<&str, StatusEntry>,
    do_parse!(
        staged_status: parse_status >>
        unstaged_status: parse_status >>
        char!(' ') >>
        submodule_status: parse_maybe_submodule_status >>
        char!(' ') >>
        file_mode: parse_file_mode >>
        char!(' ') >>
        oids: parse_status_oids >>
        char!(' ') >>
        score: parse_score >>
        char!(' ') >>
        path: take_until!("\t") >>
        char!('\t') >>
        original_path: take_until!("\n") >>
        (StatusEntry::CopiedOrRenamedStatusEntry(CopiedOrRenamedStatusEntry {
            staged_status,
            unstaged_status,
            submodule_status,
            file_mode,
            oids,
            score,
            path: path.to_string(),
            original_path: original_path.to_string(),
        }))
    )
);

named!(parse_unmerged_status_entry<&str, StatusEntry>,
    do_parse!(
        staged_status: parse_status_unmerged >>
        unstaged_status: parse_status_unmerged >>
        char!(' ') >>
        submodule_status: parse_maybe_submodule_status >>
        char!(' ') >>
        file_mode: parse_unmerged_file_mode >>
        char!(' ') >>
        oids: parse_unmerged_status_oids >>
        char!(' ') >>
        path: take_until!("\n") >>
        (StatusEntry::UnmergedStatusEntry(UnmergedStatusEntry {
            staged_status,
            unstaged_status,
            submodule_status,
            file_mode,
            oids,
            path: path.to_string(),
        }))
    )
);

named!(parse_untracked_status_entry<&str, StatusEntry>,
    do_parse!(
        path: take_until!("\n") >>
        (StatusEntry::UntrackedStatusEntry(UntrackedStatusEntry { path: path.to_string() }))
    )
);

named!(parse_ignored_status_entry<&str, StatusEntry>,
    do_parse!(
        path: take_until!("\n") >>
        (StatusEntry::IgnoredStatusEntry(IgnoredStatusEntry { path: path.to_string() }))
    )
);

named!(parse_status_entry<&str, StatusEntry>,
    switch!(take!(2),
        "1 " => call!(parse_ordinary_status_entry) |
        "2 " => call!(parse_copied_or_renamed_status_entry) |
        "u " => call!(parse_unmerged_status_entry) |
        "! " => call!(parse_untracked_status_entry) |
        "? " => call!(parse_ignored_status_entry)
    )
);

named!(parse_status_entries<&str, Vec<StatusEntry>>,
    do_parse!(
        entries: separated_list!(
            char!('\n'),
            complete!(parse_status_entry)
        ) >>
        char!('\n') >>
        (entries)
    )
);

#[derive(Debug, Serialize)]
pub struct AncestorSide {
    file_mode: u32,
    oid: String,
}

#[derive(Debug, Serialize)]
pub struct ConflictSide {
    file_mode: u32,
    oid: String,
    status: Status,
}

#[derive(Debug, Serialize)]
pub struct ConflictStatusEntry {
    ancestor: AncestorSide,
    our: ConflictSide,
    path: String,
    submodule_status: Option<SubmoduleStatus>,
    their: ConflictSide,
    worktree_file_mode: u32,
}

impl ConflictStatusEntry {
    fn from_unmerged_status_entry(entry: &UnmergedStatusEntry) -> StatusEntryOutput {
        StatusEntryOutput::Conflict(Self {
            ancestor: AncestorSide {
                file_mode: entry.file_mode.stage_1.clone(),
                oid: entry.oids.stage_1.clone(),
            },
            our: ConflictSide {
                file_mode: entry.file_mode.stage_2.clone(),
                oid: entry.oids.stage_2.clone(),
                status: entry.staged_status.clone(),
            },
            path: entry.path.clone(),
            submodule_status: entry.submodule_status.clone(),
            their: ConflictSide {
                file_mode: entry.file_mode.stage_3.clone(),
                oid: entry.oids.stage_3.clone(),
                status: entry.unstaged_status.clone(),
            },
            worktree_file_mode: entry.file_mode.worktree.clone(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct StagedStatusEntry {
    file_mode: u32,
    oids: StatusOids,
    original_path: Option<String>,
    path: String,
    score: Option<Score>,
    status: Status,
    submodule_status: Option<SubmoduleStatus>,
}

impl StagedStatusEntry {
    fn from_ordinary_status_entry(entry: &OrdinaryStatusEntry) -> Option<StatusEntryOutput> {
        entry.staged_status.as_ref().map(|status| {
            StatusEntryOutput::Staged(Self {
                file_mode: entry.file_mode.index.clone(),
                oids: entry.oids.clone(),
                original_path: None,
                path: entry.path.clone(),
                score: None,
                status: status.clone(),
                submodule_status: entry.submodule_status.clone().map(|_| SubmoduleStatus {
                    commit_changed: false,
                    has_tracked_changes: false,
                    has_untracked_changes: false,
                }),
            })
        })
    }

    fn from_copied_or_renamed_status_entry(entry: &CopiedOrRenamedStatusEntry) -> Option<StatusEntryOutput> {
        entry.staged_status.as_ref().map(|status| {
            StatusEntryOutput::Staged(Self {
                file_mode: entry.file_mode.index.clone(),
                oids: entry.oids.clone(),
                original_path: Some(entry.original_path.clone()),
                path: entry.path.clone(),
                score: Some(entry.score.clone()),
                status: status.clone(),
                submodule_status: entry.submodule_status.clone().map(|_| SubmoduleStatus {
                    commit_changed: false,
                    has_tracked_changes: false,
                    has_untracked_changes: false,
                }),
            })
        })
    }
}

#[derive(Debug, Serialize)]
pub struct UnstagedStatusEntry {
    file_mode: Option<u32>,
    oids: Option<StatusOids>,
    path: String,
    status: Status,
    submodule_status: Option<SubmoduleStatus>,
}

impl UnstagedStatusEntry {
    fn from_ordinary_status_entry(entry: &OrdinaryStatusEntry) -> Option<StatusEntryOutput> {
        entry.unstaged_status.as_ref().map(|status| {
            StatusEntryOutput::Unstaged(Self {
                file_mode: Some(entry.file_mode.worktree.clone()),
                oids: Some(entry.oids.clone()),
                path: entry.path.clone(),
                status: status.clone(),
                submodule_status: entry.submodule_status.clone(),
            })
        })
    }

    fn from_copied_or_renamed_status_entry(entry: &CopiedOrRenamedStatusEntry) -> Option<StatusEntryOutput> {
        entry.unstaged_status.as_ref().map(|status| {
            StatusEntryOutput::Unstaged(Self {
                file_mode: Some(entry.file_mode.worktree.clone()),
                oids: Some(entry.oids.clone()),
                path: entry.path.clone(),
                status: status.clone(),
                submodule_status: entry.submodule_status.clone(),
            })
        })
    }

    fn from_untracked_status_entry(entry: &UntrackedStatusEntry) -> StatusEntryOutput {
        StatusEntryOutput::Unstaged(Self {
            file_mode: None,
            oids: None,
            path: entry.path.clone(),
            status: Status::Added,
            submodule_status: None,
        })
    }
}

pub enum StatusEntryOutput {
    Conflict(ConflictStatusEntry),
    Ignored(IgnoredStatusEntry),
    Staged(StagedStatusEntry),
    Unstaged(UnstagedStatusEntry),
}

#[derive(Debug, Default, Serialize)]
pub struct StatusResult {
    conflicts: Vec<ConflictStatusEntry>,
    ignored: Vec<IgnoredStatusEntry>,
    staged: Vec<StagedStatusEntry>,
    unstaged: Vec<UnstagedStatusEntry>,
}

impl StatusResult {
    fn new() -> StatusResult {
        Default::default()
    }
}

pub fn build_git_status_output(entries: Vec<StatusEntry>) -> StatusResult {
    entries
        .iter()
        .map(|entry| -> Vec<Option<StatusEntryOutput>> {
            match entry {
                StatusEntry::OrdinaryStatusEntry(entry) => vec![
                    StagedStatusEntry::from_ordinary_status_entry(entry),
                    UnstagedStatusEntry::from_ordinary_status_entry(entry),
                ],
                StatusEntry::CopiedOrRenamedStatusEntry(entry) => vec![
                    StagedStatusEntry::from_copied_or_renamed_status_entry(entry),
                    UnstagedStatusEntry::from_copied_or_renamed_status_entry(entry),
                ],
                StatusEntry::UnmergedStatusEntry(entry) => {
                    vec![Some(ConflictStatusEntry::from_unmerged_status_entry(entry))]
                }
                StatusEntry::UntrackedStatusEntry(entry) => {
                    vec![Some(UnstagedStatusEntry::from_untracked_status_entry(entry))]
                }
                StatusEntry::IgnoredStatusEntry(entry) => {
                    // this is not ideal
                    vec![Some(StatusEntryOutput::Ignored(entry.clone()))]
                }
            }
        })
        .flat_map(|entry| entry)
        .filter_map(|option| option)
        .fold(StatusResult::new(), |mut status_result, entry| {
            match entry {
                StatusEntryOutput::Conflict(conflict) => {
                    status_result.conflicts.push(conflict);
                }
                StatusEntryOutput::Ignored(ignored) => {
                    status_result.ignored.push(ignored);
                }
                StatusEntryOutput::Staged(staged) => {
                    status_result.staged.push(staged);
                }
                StatusEntryOutput::Unstaged(unstaged) => {
                    status_result.unstaged.push(unstaged);
                }
            };
            status_result
        })
}

pub fn parse_git_status(input: &str) -> Result<StatusResult, Error> {
    parse_status_entries(input)
        .map_err(|_| Error::Process(Parsing))
        .map(|(_, vec)| build_git_status_output(vec))
}
