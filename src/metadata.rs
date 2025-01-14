use anyhow::{anyhow, Context};
use cargo_metadata::{Metadata, MetadataCommand};
use std::fs;
use std::path::{Path, PathBuf};

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

/// プロジェクトのメタデータを取得する
pub(crate) fn manifest_to_metadata(path: &Path) -> anyhow::Result<Metadata> {
  let mut cmd = MetadataCommand::new();
  cmd.manifest_path(path);
  let metadata = cmd
    .exec()
    .with_context(|| format!("Faild to get metadata for {}", path.display()))?;
  Ok(metadata)
}
