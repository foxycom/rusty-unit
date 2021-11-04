use proc_macro2::Span;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path::Path;
use crate::branch::Branch;
use crate::fitness::FitnessValue;


lazy_static! {
    static ref ROOT_REGEX: Regex = Regex::new(r"root\[.+, (?P<branch_id>\d+)\]").unwrap();
    static ref DECISION_REGEX: Regex =
        Regex::new(r"branch\[(?P<branch_id>\d+), (?P<other_branch_id>\d+), (?P<distance>\d+)\]")
            .unwrap();
}

pub struct TraceParser {}

impl TraceParser {
    pub fn parse(path: &str) -> Result<HashMap<Branch, FitnessValue>, io::Error> {
        //let mut coverage = HashMap::new();

        let mut state = State::None;
        match TraceParser::lines(path) {
            Ok(lines) => {
                for line in lines {
                    if let Ok(trace_line) = line {
                        if trace_line.starts_with(">>") {

                        }
                    }
                }

                Ok(coverage)
            }
            Err(err) => Err(err)
        }
        todo!()
    }

    fn lines<P>(path: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
    where
        P: AsRef<Path>,
    {
        let file = fs::File::open(path)?;
        Ok(io::BufReader::new(file).lines())
    }

    fn parse_line(line: &str, state: State) -> Line {
        /*return if line.starts_with("root") {
            let cap = ROOT_REGEX.captures(line)?;
            Some(TraceData {
                branch_type: BranchType::Root,
                branch_id: cap["branch_id"].parse::<u64>().unwrap(),
                other_branch_id: None,
                distance: None,
            })
        } else {
            let cap = DECISION_REGEX.captures(line)?;

            Some(TraceData {
                branch_type: BranchType::Decision,
                branch_id: cap["branch_id"].parse::<u64>().unwrap(),
                other_branch_id: cap["other_branch_id"].parse::<u64>().ok(),
                distance: cap["distance"].parse::<f64>().ok(),
            })
        };*/
        todo!()
    }

    fn parse_method_desc(line: &str) ->
}

enum State {
    Branches,
    Locals,
    CDG,
    BasicBlocks,
    None
}

pub enum Line {
    Branch(BranchData),
    Local(LocalData),
    CDG,
    BasicBlock(BasicBlockData)
}
struct BranchData {
    branch_id: u64,
    other_branch_id: Option<u64>,
    distance: Option<f64>,
}

struct LocalData {

}

struct BasicBlockData {

}

#[cfg(test)]
mod tests {
    use generation::parser::TraceParser;
    use std::collections::HashMap;


}
