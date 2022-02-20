
pub fn rustc_get_crate_name(rustc_args: &[String]) -> String {
    let pos = rustc_args.iter().position(|a| a == "--crate-name").unwrap();
    rustc_args.get(pos + 1).map(|s| s.to_string()).unwrap()
}
