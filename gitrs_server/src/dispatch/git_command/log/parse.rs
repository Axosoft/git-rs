use error::protocol::{Error, ProcessError::Parsing};
use util::parse::{sha, short_sha};

#[derive(Debug, Serialize)]
pub struct LogEntry {
    commit_sha: String,
    merge: Option<(String, String)>,
    author: String,
    date: String,
    commit_message: String,
}

named!(pub parse_merge<&str, (String, String)>,
    do_parse!(
        tag!("Merge: ") >>
        left: short_sha >>
        char!(' ') >>
        right: short_sha >>
        char!('\n') >>
        ((String::from(left), String::from(right)))
    )
);

/*
* Each indent starts with 4 spaces
* seperator line with no spaces
*/
named!(pub parse_log_entry<&str, LogEntry>,
    do_parse!(
        tag!("commit ") >>
        commit_sha: sha >>
        char!('\n') >>
        merge: opt!(parse_merge) >>
        tag!("Author: ") >>
        author: take_until!("\n") >>
        char!('\n') >>
        tag!("Date:   ") >>
        date: take_until!("\n") >>
        tag!("\n\n    ") >>
        commit_message: take_until!("\n\n") >>
        (LogEntry {
            commit_sha: String::from(commit_sha),
            merge,
            author: String::from(author),
            date: String::from(date),
            commit_message: String::from(commit_message),
        })
    )
);

named!(pub parse_log_entries<&str, Vec<LogEntry>>,
    do_parse!(
        entries: separated_list!(
            tag!("\n\n"),
            complete!(parse_log_entry)
        ) >>
        (entries)
    )
);

pub fn parse_log(input: &str) -> Result<Vec<LogEntry>, Error> {
    let mut input = String::from(input);
    input.push('\n');

    parse_log_entries(&input)
        .map_err(|_| Error::Process(Parsing))
        .map(|(_, vec)| vec)
}
