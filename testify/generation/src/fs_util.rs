use std::io::ErrorKind;
use std::path::Path;
use std::process;

pub fn remove_dir_all(path: &Path) -> std::io::Result<()> {
    if let Err(err) = std::fs::remove_dir_all(path) {
        if err.kind() != ErrorKind::NotFound {
            return Err(err);
        }
    }

    Ok(())
}

pub fn sysroot() -> String {
    let out = process::Command::new("rustc")
        .arg("--print=sysroot")
        .current_dir(".")
        .output()
        .unwrap();

    let sysroot = String::from_utf8(out.stdout).unwrap();
    let sysroot = sysroot.trim();
    sysroot.to_string()
}