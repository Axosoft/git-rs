use nom::{line_ending, rest, digit1, oct_digit1};
use std::str;

fn parse_num(input: &str, radix: u32) -> u32 {
    u32::from_str_radix(input, radix).unwrap()
}

#[derive(Debug, Serialize)]
pub enum Status {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
}

named!(parse_status<&str, Option<Status>>,
    do_parse!(
        status: switch!(take!(1),
            "M" => value!(Some(Status::Modified)) |
            "A" => value!(Some(Status::Added)) |
            "D" => value!(Some(Status::Deleted)) |
            "R" => value!(Some(Status::Renamed)) |
            "C" => value!(Some(Status::Copied)) |
            "?" => value!(Some(Status::Untracked)) |
            "." => value!(None)
        ) >>
        (status)
    )
);

#[derive(Debug, Serialize)]
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
            head: parse_num(head, 8),
            index: parse_num(index, 8),
            worktree: parse_num(worktree, 8)
        })
    )
);

#[derive(Debug, Serialize)]
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
            stage_1: parse_num(stage_1, 8),
            stage_2: parse_num(stage_2, 8),
            stage_3: parse_num(stage_3, 8),
            worktree: parse_num(worktree, 8)
        })
    )
);

#[derive(Debug, Serialize)]
pub enum ScoreType {
    Renamed,
    Copied,
}

#[derive(Debug, Serialize)]
pub struct Score {
    score_type: ScoreType,
    percentage: u32,
}

named!(parse_score<&str, Score>,
    do_parse!(
        score_type: switch!(take!(1), "R" => value!(ScoreType::Renamed) | "C" => value!(ScoreType::Copied)) >>
        percentage: digit1 >>
        (Score { score_type, percentage: parse_num(percentage, 10) })
    )
);

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct StatusOids {
    head: String,
    index: String,
}

named!(parse_status_oids<&str, StatusOids>,
    do_parse!(
        head: take!(40) >>
        char!(' ') >>
        index: take!(40) >>
        (StatusOids { head: head.to_string(), index: index.to_string() })
    )
);

#[derive(Debug, Serialize)]
pub struct UnmergedStatusOids {
    stage_1: String,
    stage_2: String,
    stage_3: String,
}

named!(parse_unmerged_status_oids<&str, UnmergedStatusOids>,
    do_parse!(
        stage_1: take!(40) >>
        char!(' ') >>
        stage_2: take!(40) >>
        char!(' ') >>
        stage_3: take!(40) >>
        (UnmergedStatusOids {
            stage_1: stage_1.to_string(),
            stage_2: stage_2.to_string(),
            stage_3: stage_3.to_string()
        })
    )
);

#[derive(Debug, Serialize)]
pub enum StatusEntry {
    OrdinaryStatusEntry(OrdinaryStatusEntry),
    CopiedOrRenamedStatusEntry(CopiedOrRenamedStatusEntry),
    UnmergedStatusEntry(UnmergedStatusEntry),
    UntrackedStatusEntry(UntrackedStatusEntry),
    IgnoredStatusEntry(IgnoredStatusEntry),
}

#[derive(Debug, Serialize)]
pub struct OrdinaryStatusEntry {
    staged_status: Option<Status>,
    unstaged_status: Option<Status>,
    submodule_status: Option<SubmoduleStatus>,
    file_mode: FileModeStatus,
    oids: StatusOids,
    path: String,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct UnmergedStatusEntry {
    staged_status: Option<Status>,
    unstaged_status: Option<Status>,
    submodule_status: Option<SubmoduleStatus>,
    file_mode: UnmergedFileModeStatus,
    oids: UnmergedStatusOids,
    path: String,
}

#[derive(Debug, Serialize)]
pub struct UntrackedStatusEntry {
    path: String,
}

#[derive(Debug, Serialize)]
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
        path: take_until_and_consume!("\n") >>
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
        path: take_until_and_consume!("\t") >>
        original_path: take_until_and_consume!("\n") >>
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
        staged_status: parse_status >>
        unstaged_status: parse_status >>
        char!(' ') >>
        submodule_status: parse_maybe_submodule_status >>
        char!(' ') >>
        file_mode: parse_unmerged_file_mode >>
        char!(' ') >>
        oids: parse_unmerged_status_oids >>
        char!(' ') >>
        path: take_until_and_consume!("\n") >>
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
        path: take_until_and_consume!("\n") >>
        (StatusEntry::UntrackedStatusEntry(UntrackedStatusEntry { path: path.to_string() }))
    )
);

named!(parse_ignored_status_entry<&str, StatusEntry>,
    do_parse!(
        path: take_until_and_consume!("\n") >>
        (StatusEntry::IgnoredStatusEntry(IgnoredStatusEntry { path: path.to_string() }))
    )
);

named!(parse_status_entry<&str, StatusEntry>,
    switch!(take!(2),
        "1 " => call!(parse_ordinary_status_entry) |
        "2 " => call!(parse_copied_or_renamed_status_entry) |
        "u " => call!(parse_unmerged_status_entry) |
        "! " => call!(parse_untracked_status_entry) |
        "? " => complete!(call!(parse_ignored_status_entry))
    )
);

named!(pub parse_status_entries<&str, Vec<StatusEntry>>,
    do_parse!(
        entries: many_till!(parse_status_entry, char!('\0')) >>
        (entries.0)
    )
);
