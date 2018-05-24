use nom::digit1;
use util::parse::{sha, parse_u32};

#[derive(Debug, Serialize)]
pub struct BisectStep {
    current_commit_sha: String,
    num_revisions_left: u32,
    num_steps_left: u32,
}

#[derive(Debug, Serialize)]
pub struct BisectFinish {
    bad_commit_sha: String,
}

#[derive(Debug, Serialize)]
pub enum BisectOutput {
    Step(BisectStep),
    Finish(BisectFinish),
}

named!(parse_bisect_step<&str, BisectStep>,
    do_parse!(
        num_revisions_left: digit1 >>
        tag!(" revision") >>
        opt!(char!('s')) >>
        tag!(" left to test after this (roughly ") >>
        num_steps_left: digit1 >>
        tag!(" step") >>
        opt!(char!('s')) >>
        tag!(")\n[") >>
        current_commit_sha: sha >>
        (BisectStep {
            current_commit_sha: String::from(current_commit_sha),
            num_revisions_left: parse_u32(num_revisions_left, 10),
            num_steps_left: parse_u32(num_steps_left, 10),
        })
    )
);

named!(parse_bisect_success<&str, BisectFinish>,
    do_parse!(
        bad_commit_sha: sha >>
        (BisectFinish {
            bad_commit_sha: String::from(bad_commit_sha),
        })
    )
);

named!(pub parse_bisect<&str, BisectOutput>,
    switch!(opt!(tag!("Bisecting: ")),
        Some("Bisecting: ") => dbg!(map!(call!(parse_bisect_step), BisectOutput::Step)) |
        None => map!(call!(parse_bisect_success), BisectOutput::Finish)
    )
);
