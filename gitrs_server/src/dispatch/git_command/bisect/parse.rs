use nom::digit1;
use util::parse::{parse_u32, sha};

#[derive(Debug, Serialize)]
pub struct BisectStep {
    current_commit_sha: String,
    num_revisions_left: u32,
    num_steps_left: u32,
}

named!(pub parse_bisect_step<&str, BisectStep>,
    do_parse!(
        tag!("Bisecting: ") >>
        num_revisions_left: digit1 >>
        tag!(" revision") >>
        opt!(char!('s')) >>
        tag!("left to test after this (roughly ") >>
        num_steps_left: digit1 >>
        tag!(" steps)\n[") >>
        current_commit_sha: sha >>
        (BisectStep {
            current_commit_sha: String::from(current_commit_sha),
            num_revisions_left: parse_u32(num_revisions_left, 10),
            num_steps_left: parse_u32(num_steps_left, 10),
        })
    )
);
