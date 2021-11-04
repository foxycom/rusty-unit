use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct InstrumentationWriter {
    file: Option<File>,
}

impl InstrumentationWriter {
    pub fn new<P>(path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.into().as_path())
            .unwrap();
        InstrumentationWriter { file: Some(file) }
    }

    pub fn new_body(&mut self, id: &str) {
        self.file
            .as_mut()
            .unwrap()
            .write_all(format!(">>{}\n", id).as_bytes())
            .unwrap();
    }

    pub fn write_cdg(&mut self, cdg: &str) {
        let file = self.file.as_mut().unwrap();
        file.write_all(format!("#cdg\n{}\n", cdg).as_bytes())
            .unwrap();
    }

    pub fn write_locals(&mut self, locals: &Vec<String>) {
        let file = self.file.as_mut().unwrap();
        file.write_all(b"#locals\n").unwrap();
        for local in locals {
            file.write_all(format!("{}\n", local).as_bytes()).unwrap();
        }
    }

    pub fn write_branches(&mut self, branches: &Vec<String>) {
        let file = self.file.as_mut().unwrap();
        file.write_all(b"#branches\n").unwrap();
        for branch in branches {
            file.write_all(format!("{}\n", branch).as_bytes()).unwrap();
        }
    }

    pub fn write_basic_blocks(&mut self, basic_blocks: &Vec<String>) {
        let file = self.file.as_mut().unwrap();
        file.write_all(b"#basic_blocks\n").unwrap();
        for block in basic_blocks {
            file.write_all(format!("<data>{}\n", block).as_bytes()).unwrap();
        }
    }
}
