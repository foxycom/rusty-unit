#[derive(Eq, PartialEq)]
pub enum Stage {
    Analyze,
    Instrument,
}

pub fn get_stage(args: &[String]) -> Stage {
    let stage_arg = args.iter().find(|&a| a.starts_with("--stage"));

    if let Some(stage_arg) = stage_arg {
        let stage = stage_arg.split("=").last().unwrap();
        Stage::from(stage)
    } else {
        Stage::Instrument
    }
}

pub fn get_crate_root(args: &[String]) -> String {
    let crate_arg = args.iter().find(|&a| a.starts_with("--crate"));
    if let Some(crate_root) = crate_arg {
        let crate_root = crate_root.split("=").last().unwrap();
        crate_root.to_owned()
    } else {
        panic!("Crate root is not specified in the compiler args");
    }
}

pub fn get_testify_flags() -> Vec<String> {
    let var = std::env::var("TESTIFY_FLAGS").expect("TESTIFY_FLAGS is not set");
    var.split(" ").map(|a| a.to_owned()).collect()
}

pub fn get_cut_name(args: &[String]) -> String {
    let cut_name_opt = args.iter().find(|&a| a.starts_with("--crate-name"));
    if let Some(cut_name_opt) = cut_name_opt {
        let cut_name = cut_name_opt.split("=").last().unwrap();
        cut_name.to_owned()
    } else {
        panic!("Name of the crate under test is not specified");
    }
}