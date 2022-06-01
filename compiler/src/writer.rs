use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{PathBuf, Path};
use rustc_middle::ty::subst::Subst;
use crate::{HIR_LOG_PATH, INSTRUMENTED_MIR_LOG_NAME, LOG_DIR, LOG_EXT, MIR_LOG_NAME, RuConfig};
use serde::Serialize;
use crate::types::{RuCallable, RuConstVal, RuTrait};

#[derive(Builder, Serialize)]
pub struct MirObject {
  global_id: String,
  #[cfg(feature = "analysis")]
  cdg: String,
  #[cfg(feature = "analysis")]
  cdg_dot: String,
  #[cfg(feature = "analysis")]
  #[builder(default)]
  cfg: String,
  #[builder(default)]
  #[cfg(feature = "analysis")]
  truncated_cfg: String,
  #[builder(default)]
  #[cfg(feature = "analysis")]
  constant_pool: Vec<RuConstVal>,
  #[builder(default)]
  #[cfg(feature = "analysis")]
  branches: u64,
  #[builder(default)]
  #[cfg(feature = "analysis")]
  assertions: u64,
  locals: Vec<String>,
  basic_blocks: Vec<String>,
}

#[derive(Builder, Serialize)]
pub struct HirObject {
  name: String,
  callables: Vec<RuCallable>,
  // Types that implement traits
  impls: HashMap<String, Vec<String>>
}

#[cfg(feature = "analysis")]
pub struct MirWriter {}

#[cfg(feature = "analysis")]
impl MirWriter {
  pub fn write(mir_object: &MirObject) {
    let mut hasher = DefaultHasher::new();
    let hash = mir_object.global_id.hash(&mut hasher);

    let file_name = format!("{}_{}.{}", MIR_LOG_NAME, &mir_object.global_id, LOG_EXT);

    let path = Path::new(&RuConfig::env_crate_root()).join("analysis").join("mir").join("original").join(file_name);

    #[cfg(feature = "file_writer")]
    file_writer::Writer::write(path.as_path(), serde_json::to_string(mir_object).as_ref().unwrap());
    #[cfg(feature = "redis_writer")]
    redis_writer::Writer::write(path.as_path(), serde_json::to_string(mir_object).as_ref().unwrap());
  }

  pub fn write_instrumented(mir_object: &MirObject) {
    let file_name = format!("{}_{}.{}", INSTRUMENTED_MIR_LOG_NAME, &mir_object.global_id, LOG_EXT);
    let path = Path::new(&RuConfig::env_crate_root()).join("analysis").join("mir").join("instrumented").join(file_name);
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
    let path = Path::new(&RuConfig::env_crate_root()).join("analysis").join("hir").join(format!("{}.{}", HIR_LOG_PATH, LOG_EXT));

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
      // Create parent directories
      let parent = path.as_ref().parent().unwrap();
      std::fs::create_dir_all(parent).unwrap();

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
  use std::path::Path;

  pub(super) struct Writer {}

  impl Writer {
    pub(super) fn write(path: impl AsRef<Path>, content: &str) {
      todo!()
    }
  }
}
