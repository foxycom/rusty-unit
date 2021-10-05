use crate::chromosome::FitnessValue;
use crate::source::{Branch, BranchType};
use proc_macro2::Span;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path::Path;

lazy_static! {
    static ref ROOT_REGEX: Regex = Regex::new(r"root\[.+, (?P<branch_id>\d+)\]").unwrap();
    static ref DECISION_REGEX: Regex =
        Regex::new(r"branch\[(?P<branch_id>\d+), (?P<other_branch_id>\d+), (?P<distance>\d+)\]")
            .unwrap();
}

pub struct TraceParser {}

impl TraceParser {
    pub fn parse(path: &str) -> Result<HashMap<Branch, FitnessValue>, io::Error> {
        let mut coverage = HashMap::new();

        match TraceParser::lines(path) {
            Ok(lines) => {
                for line in lines {
                    if let Ok(trace_line) = line {
                        let data = TraceParser::parse_line(&trace_line)
                            .ok_or(io::Error::new(io::ErrorKind::Other, "Could not read data"))?;
                        coverage.insert(
                            Branch::new(data.branch_id, data.branch_type.clone(), Span::call_site()),
                            FitnessValue::Zero,
                        );
                        if let Some(other_branch) = data.other_branch_id {
                            let dist = data.distance.ok_or(io::Error::new(
                                io::ErrorKind::Other,
                                "No distance to other branch known",
                            ))?;
                            coverage.insert(
                                Branch::new(other_branch, data.branch_type, Span::call_site()),
                                FitnessValue::Val(dist),
                            );
                        }
                    }
                }

                Ok(coverage)
            }
            Err(err) => Err(err)
        }

    }

    fn lines<P>(path: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
    where
        P: AsRef<Path>,
    {
        let file = fs::File::open(path)?;
        Ok(io::BufReader::new(file).lines())
    }

    fn parse_line(line: &str) -> Option<TraceData> {
        return if line.starts_with("root") {
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
        };
    }
}

struct TraceData {
    branch_type: BranchType,
    branch_id: u64,
    other_branch_id: Option<u64>,
    distance: Option<f64>,
}

#[cfg(test)]
mod tests {
    use crate::parser::TraceParser;
    use std::collections::HashMap;

    #[test]
    fn test_parse_trace() {
        let mut expected = HashMap::new();
        expected.insert(2u64, 109.0);
        expected.insert(6u64, 0.0);
        expected.insert(3u64, 0.0);
        let path = "/Users/tim/Documents/master-thesis/testify/src/examples/additions/trace.txt";
        let results = TraceParser::parse(path).unwrap();
        assert_eq!(expected, results);
    }
}
