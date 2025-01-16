use anyhow::{anyhow, Context};
use cargo_manifest::Manifest;
use cargo_metadata::{Metadata, MetadataCommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Cargo.tomlファイルを探す
pub(crate) fn find_manifest_file(dir: &Path) -> anyhow::Result<(usize, PathBuf)> {
  let manifest_file_name = "Cargo.toml";
  // 見つからなかったら階層を親側に辿る
  for (i, path) in dir.ancestors().enumerate() {
    let f = path.join(manifest_file_name);
    if let Ok(true) = fs::exists(&f) {
      return Ok((i, f));
    }
  }
  Err(anyhow!("Not find Cargo.toml for {}", dir.display()))
}

pub(crate) fn get_manifest_data_from_path(path: &Path) -> anyhow::Result<Manifest> {
  let manifest = Manifest::from_path(path)?;
  Ok(manifest)
}

pub(crate) fn get_manifest_data_from_contents(contents: &str) -> anyhow::Result<Manifest> {
  let manifest = Manifest::from_str(contents)?;
  Ok(manifest)
}

pub(crate) fn lib_file_path(manifest: &Manifest) -> String {
  if let Some(lib) = &manifest.lib {
    if let Some(lib_path) = &lib.path {
      lib_path.clone()
    } else {
      "src/lib.rs".to_string()
    }
  } else {
    "src/lib.rs".to_string()
  }
}

#[allow(dead_code)]
/// プロジェクトのメタデータを取得する
pub(crate) fn manifest_to_metadata(path: &Path) -> anyhow::Result<Metadata> {
  let mut cmd = MetadataCommand::new();
  cmd.manifest_path(path);
  let metadata = cmd
    .exec()
    .with_context(|| format!("Faild to get metadata for {}", path.display()))?;
  Ok(metadata)
}
