use util::parse::{parse_u32, sha};

#[derive(Debug, Serialize)]
pub struct StepOutput {
    current_commit_sha: String,
    num_revisions_left: u32,
    num_steps_left: u32,
}

named!(parse_start<&str, StepOutput>,
    do_parse!(
        tag!("Bisecting: ") >>
        num_revisions_left: call!(parse_u32, 10) >>
        tag!(" revision") >>
        opt!(char!('s')) >>
        tag!("left to test after this (roughly ") >>
        num_steps_left: call!(parse_u32, 10) >>
        tag!(" steps)\n[") >>
        current_commit_sha: sha >>
        (StepOutput {
            current_commit_sha: String::from(current_commit_sha),
            num_revisions_left,
            num_steps_left,
        })
    )
);
