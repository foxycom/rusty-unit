use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{PathBuf, Path};
use crate::{HIR_LOG_PATH, INSTRUMENTED_MIR_LOG_NAME, LOG_DIR, LOG_EXT, MIR_LOG_NAME};
use serde::Serialize;
use crate::types::{Callable, Trait};

#[derive(Builder, Serialize)]
pub struct MirObject {
  global_id: String,
  #[cfg(feature = "analysis")]
  cdg: String,
  #[cfg(feature = "analysis")]
  cfg: String,
  #[cfg(feature = "analysis")]
  truncated_cfg: String,
  locals: Vec<String>,
  basic_blocks: Vec<String>,
}

#[derive(Builder, Serialize)]
pub struct HirObject {
  callables: Vec<Callable>,
  // Types that implement traits
  impls: std::collections::HashMap<String, Vec<String>>
}

#[cfg(feature = "analysis")]
pub struct MirWriter {}

#[cfg(feature = "analysis")]
impl MirWriter {
  pub fn write(mir_object: &MirObject) {
    let file_name = format!("{}_{}.{}", MIR_LOG_NAME, &mir_object.global_id, LOG_EXT);
    let path = Path::new(LOG_DIR).join(file_name);

    #[cfg(feature = "file_writer")]
    file_writer::Writer::write(path.as_path(), serde_json::to_string(mir_object).as_ref().unwrap());
    #[cfg(feature = "redis_writer")]
    redis_writer::Writer::write(path.as_path(), serde_json::to_string(mir_object).as_ref().unwrap());
  }

  pub fn write_instrumented(mir_object: &MirObject) {
    let file_name = format!("{}_{}.{}", INSTRUMENTED_MIR_LOG_NAME, &mir_object.global_id, LOG_EXT);
    let path = Path::new(LOG_DIR).join(file_name);
    #[cfg(feature = "file_writer")]
    file_writer::Writer::write(path.as_path(), serde_json::to_string(mir_object).as_ref().unwrap());
    #[cfg(feature = "redis_writer")]
    redis_writer::Writer::write(path.as_path(), serde_json::to_string(mir_object).as_ref().unwrap());

  }
}

#[cfg(feature = "analysis")]
pub struct HirWriter {}

#[cfg(feature = "analysis")]
impl HirWriter {
  pub fn write(hir_object: &HirObject) {
    let path = Path::new(LOG_DIR).join(format!("{}.{}", HIR_LOG_PATH, LOG_EXT));

    #[cfg(feature = "file_writer")]
    file_writer::Writer::write(path.as_path(), serde_json::to_string(hir_object).as_ref().unwrap());
    #[cfg(feature = "redis_writer")]
    redis_writer::Writer::write(path.as_path(), serde_json::to_string(hir_object).as_ref().unwrap());
  }
}

#[cfg(feature = "file_writer")]
mod file_writer {
  use super::*;
  pub(super) struct Writer {}

  impl Writer {
    pub(super) fn write(path: impl AsRef<Path>, content: &str) {
      let mut file = OpenOptions::new()
          .create(true)
          .append(true)
          .open(path.as_ref())
          .unwrap();
      file.write_all(content.as_bytes()).unwrap();
    }
  }
}

#[cfg(feature = "redis_writer")]
mod redis_writer {
  pub(super) struct Writer {}

  impl Writer {
    pub(super) fn write(path: impl AsRef<Path>, content: &str) {
      todo!()
    }
  }
}
