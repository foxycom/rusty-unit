use std::path::Path;

pub fn instrumented_path(original_path: &str) -> String {
    let path = Path::new(original_path);
    let dir = path.parent().expect("No dir available");
    let file_name = path.file_stem().expect("No file name available");

    let new_file_name = format!("{}_instrumented.rs", file_name.to_str().unwrap());
    let new_path = dir.join(Path::new(&new_file_name));

    let str_path = new_path.to_str().unwrap().to_owned();
    str_path
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instrumented_path() {
        assert_eq!("/abc/some_file_instrumented.rs", instrumented_path("/abc/some_file.rs"));
    }
}