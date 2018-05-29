use nom::digit1;
use util::parse::{sha, parse_u32};

#[derive(Debug, Serialize)]
pub struct BisectStep {
    current_commit_sha: String,
    num_revisions_left: u32,
    num_steps_left: u32,
}

#[derive(Debug, Serialize)]
pub struct BisectReachedMergeBase {
    merge_base_sha: String,
}

#[derive(Debug, Serialize)]
pub struct BisectFoundRange {
    bad_commit_sha: String,
    good_commit_sha: String,
}

#[derive(Debug, Serialize)]
pub struct BisectFoundSingle {
    bad_commit_sha: String,
}

#[derive(Debug, Serialize)]
pub enum BisectFinish {
    FoundRange(BisectFoundRange),
    FoundSingle(BisectFoundSingle),
}

#[derive(Debug, Serialize)]
pub struct BisectVisualize {
    shas: Vec<String>,
}

#[derive(Debug, Serialize)]
pub enum BisectOutput {
    Finish(BisectFinish),
    ReachedMergeBase(BisectReachedMergeBase),
    Step(BisectStep),
    Visualize(BisectVisualize),
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

named!(parse_bisect_reached_merge_base<&str, BisectReachedMergeBase>,
    do_parse!(
        tag!("a merge base must be tested\n[") >>
        merge_base_sha: sha >>
        (BisectReachedMergeBase {
            merge_base_sha: String::from(merge_base_sha),
        })
    )
);

named!(parse_bisect_found_range<&str, BisectFoundRange>,
    do_parse!(
        tag!("merge base ") >>
        sha >>
        tag!(" is bad.\nThis means the bug has been fixed between ") >>
        bad_commit_sha: sha >>
        tag!(" and [") >>
        good_commit_sha: sha >>
        (BisectFoundRange {
            bad_commit_sha: String::from(bad_commit_sha),
            good_commit_sha: String::from(good_commit_sha),
        })
    )
);

named!(parse_bisect_found_single<&str, BisectFoundSingle>,
    do_parse!(
        bad_commit_sha: sha >>
        (BisectFoundSingle {
            bad_commit_sha: String::from(bad_commit_sha),
        })
    )
);

named!(parse_bisect_visualize<&str, Vec<String>>,
    separated_list!(
        char!('\n'),
        complete!(do_parse!(tag!("sha ") >> sha: sha >> (String::from(sha))))
    )
);

named!(pub parse_bisect<&str, BisectOutput>,
    switch!(opt!(tag!("Bisecting: ")),
        Some("Bisecting: ") => alt!(
            map!(call!(parse_bisect_step), BisectOutput::Step) |
            map!(call!(parse_bisect_reached_merge_base), BisectOutput::ReachedMergeBase)
        ) |
        None => switch!(opt!(tag!("The ")),
            Some("The ") => map!(
                call!(parse_bisect_found_range),
                { |bisect_found_range| {
                   BisectOutput::Finish(BisectFinish::FoundRange(bisect_found_range))
                } }
            ) |
            None => switch!(opt!(peek!(tag!("sha "))),
                Some("sha ") => map!(
                    call!(parse_bisect_visualize),
                    { |shas| BisectOutput::Visualize(BisectVisualize { shas }) }
                ) |
                None => map!(
                    call!(parse_bisect_found_single),
                    { |bisect_found_single| {
                        BisectOutput::Finish(BisectFinish::FoundSingle(bisect_found_single))
                    } }
                )
            )
        )
    )
);
