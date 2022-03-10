use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{PathBuf, Path};
use crate::{HIR_LOG_PATH, INSTRUMENTED_MIR_LOG_NAME, LOG_DIR, LOG_EXT, MIR_LOG_NAME};
use serde::Serialize;
use crate::types::Callable;

#[derive(Builder, Serialize)]
pub struct MirObject {
  global_id: String,
  cdg: String,
  locals: Vec<String>,
  basic_blocks: Vec<String>,
}

#[derive(Builder, Serialize)]
pub struct HirObject {
  callables: Vec<Callable>
}

pub struct MirWriter {}

impl MirWriter {
  pub fn write(mir_object: &MirObject) {
    let file_name = format!("{}_{}.{}", MIR_LOG_NAME, &mir_object.global_id, LOG_EXT);
    let path = Path::new(LOG_DIR).join(file_name);

    Writer::write(path.as_path(), serde_json::to_string(mir_object).as_ref().unwrap());
  }

  pub fn write_instrumented(mir_object: &MirObject) {
    let file_name = format!("{}_{}.{}", INSTRUMENTED_MIR_LOG_NAME, &mir_object.global_id, LOG_EXT);
    let path = Path::new(LOG_DIR).join(file_name);
    Writer::write(path.as_path(), serde_json::to_string(mir_object).as_ref().unwrap());
  }
}

pub struct HirWriter {}

impl HirWriter {
  pub fn write(hir_object: &HirObject) {
    let path = Path::new(LOG_DIR).join(format!("{}.{}", HIR_LOG_PATH, LOG_EXT));
    Writer::write(path.as_path(), serde_json::to_string(hir_object).as_ref().unwrap());
  }
}

struct Writer {}

impl Writer {
  #[cfg(file_writer)]
  fn write(path: impl AsRef<Path>, content: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path.as_ref())
        .unwrap();
    file.write_all(content.as_bytes()).unwrap();
  }

  #[cfg(redis_writer)]
  pub fn write(content: &str) {
    todo!()
  }
}
