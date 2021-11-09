use crate::analysis::{HirAnalysis, MirAnalysis, MirBody};
use crate::branch::Branch;
use crate::fitness::FitnessValue;
use petgraph::Graph;
use proc_macro2::Span;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};

lazy_static! {
    static ref ROOT_REGEX: Regex = Regex::new(r"root\[.+, (?P<branch_id>\d+)\]").unwrap();
    static ref DECISION_REGEX: Regex =
        Regex::new(r"branch\[(?P<branch_id>\d+), (?P<other_branch_id>\d+), (?P<distance>\d+)\]")
            .unwrap();
}

pub struct MirParser {}

impl MirParser {
    pub fn parse(path: &str) -> MirAnalysis {
        //let mut coverage = HashMap::new();

        let mut analysis = MirAnalysis::new();

        let mut global_id = None;
        let mut state = State::None;
        match MirParser::lines(path) {
            Ok(lines) => {
                for line in lines {
                    if let Ok(trace_line) = line {
                        if trace_line.starts_with(">>") {
                            let id = trace_line[2..].parse::<u32>().unwrap();
                            global_id = Some(id);
                        } else if trace_line.starts_with("#") {
                            let begin_state = &trace_line[1..];
                            if begin_state == "cdg" {
                                state = State::CDG(global_id.unwrap());
                            } else if begin_state == "basic_blocks" {
                                state = State::BasicBlocks(global_id.unwrap());
                            } else if begin_state == "locals" {
                                state = State::Locals(global_id.unwrap());
                            } else if begin_state == "branches" {
                                state = State::Branches(global_id.unwrap());
                            } else {
                                panic!("Undefined state");
                            }
                        } else if trace_line.starts_with("<data>") {
                            match &state {
                                State::Branches(global_id) => {
                                    let mir_body = analysis
                                        .bodies
                                        .entry(global_id.to_owned())
                                        .or_insert_with(|| MirBody::new());
                                    mir_body.branches = MirParser::parse_branches(&trace_line[6..]);
                                }
                                State::Locals(global_id) => {}
                                State::CDG(global_id) => {
                                    let mir_body = analysis
                                        .bodies
                                        .entry(global_id.to_owned())
                                        .or_insert_with(|| MirBody::new());
                                    mir_body.cdg = MirParser::parse_cdg(&trace_line[6..]);
                                }
                                State::BasicBlocks(global_id) => {}
                                State::None => panic!("State is None"),
                            }
                        } else {
                            panic!("Malformed line: {}", trace_line);
                        }
                    }
                }
            }
            _ => panic!(),
        }
        analysis
    }

    fn parse_branches(input: &str) -> Vec<Branch> {
        serde_json::from_str::<Vec<Branch>>(input).unwrap()
    }

    fn parse_cdg(input: &str) -> Graph<usize, usize> {
        serde_json::from_str::<Graph<usize, usize>>(input).unwrap()
    }

    fn lines<P>(path: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
    where
        P: AsRef<Path>,
    {
        let file = fs::File::open(path)?;
        Ok(io::BufReader::new(file).lines())
    }
}

#[derive(PartialEq, Eq)]
pub enum State {
    Branches(u32),
    Locals(u32),
    CDG(u32),
    BasicBlocks(u32),
    None,
}

pub enum Line {
    Branch(BranchData),
    Local(LocalData),
    CDG,
    BasicBlock(BasicBlockData),
}

pub struct BranchData {
    branch_id: u64,
    other_branch_id: Option<u64>,
    distance: Option<f64>,
}

pub struct LocalData {}

pub struct BasicBlockData {}

pub struct HirParser {}

impl HirParser {
    pub fn parse<P>(path: P) -> HirAnalysis
    where
        P: Into<PathBuf>,
    {
        let content = fs::read_to_string(path.into()).unwrap();
        serde_json::from_str::<HirAnalysis>(&content).unwrap()
    }
}
