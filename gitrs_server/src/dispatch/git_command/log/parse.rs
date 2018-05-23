use error::protocol::{Error, ProcessError::Parsing};
use util::parse::sha;

#[derive(Debug, Serialize)]
pub struct TreeInfo {
    sha: String,
    parents: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SignatureInfo {
    author: String,
    email: String,
    date: String,
}
#[derive(Debug, Serialize)]
pub struct BodyInfo {
    summary: String,
    description: String,
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    tree: TreeInfo,
    signature: SignatureInfo,
    body: BodyInfo,
}

named!(pub parse_parent_entries<&str, Vec<String>>, 
    switch!(peek!(take!(1)),
        "\n" => value!(Vec::new()) |
        _ => separated_list!(
            char!(' '),
            map!(sha, String::from)
        )
    )
);

named!(pub parse_tree<&str, TreeInfo>,
    do_parse!(
        tag!("sha ") >>
        sha: sha >>
        char!('\n') >>
        tag!("parents ") >>
        parents: parse_parent_entries >>
        char!('\n') >>
        (TreeInfo {
            sha: String::from(sha),
            parents: parents,
        })
    )
);

named!(pub parse_signature<&str, SignatureInfo>,
    do_parse!(
        tag!("author ") >>
        author: take_until!("\n") >>
        char!('\n') >>
        tag!("email ") >>
        email: take_until!("\n") >>
        char!('\n') >>
        tag!("date ") >>
        date: take_until!("\n") >>
        char!('\n') >>
        (SignatureInfo {
            author: String::from(author),
            email: String::from(email),
            date: String::from(date),
        })
    )
);

named!(pub parse_body<&str, BodyInfo>,
    do_parse!(
        tag!("summary ") >>
        summary: take_until!("\n") >>
        char!('\n') >>
        tag!("description ") >>
        description: take_until!("\0\0\n") >>
        (BodyInfo {
            summary: String::from(summary),
            description: String::from(description),
        })
    )
);

named!(pub parse_log_entry<&str, LogEntry>,
    do_parse!(
        tree: parse_tree >>
        signature: parse_signature >>
        body: parse_body >>
        (LogEntry {
            tree: tree,
            signature: signature,
            body: body,
        })
    )
);

named!(pub parse_log_entries<&str, Vec<LogEntry>>,
    do_parse!(
        entries: separated_list!(
            tag!("\0\0\n"),
            complete!(parse_log_entry)
        ) >>
        (entries)
    )
);

pub fn parse_log(input: &str) -> Result<Vec<LogEntry>, Error> {
    let mut input = String::from(input);

    parse_log_entries(&input)
        .map_err(|_| Error::Process(Parsing))
        .map(|(_, vec)| vec)
}
