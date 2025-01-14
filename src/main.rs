use anyhow::{anyhow, Context};
use clap::Parser;
use std::fs;
use std::path::Path;

mod file;
mod git;
mod metadata;

#[derive(Debug, Clone, Parser)]
struct Args {
  /// Path to working directory
  #[clap(short, long)]
  pub dir: Option<String>,
  /// Git Object ID
  #[clap(short, long)]
  pub oid: Option<String>,
}

fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let dir = args.dir.unwrap_or(".".to_string());
  let dir = Path::new(&dir);

  // Cargo.tomlから情報を取る
  let (manifest_dep, manifest_file) = metadata::find_manifest_file(dir)?;
  let metadata = metadata::manifest_to_metadata(&manifest_file)?;

  // git repositoryから比較対象のファイルを取れるようにする
  let (git_repo_dep, git_repo) =
    git::find_repo(dir).with_context(|| anyhow!("This Rust project is not managed by git"))?;
  if git_repo_dep > manifest_dep {
    // Cargo.tomlよりも下の階層にgitリポジトリがある状態
    // 対処できないのでエラー
    return Err(anyhow!("This Rust project is not managed by git"));
  }
  let commit = if let Some(oid_str) = args.oid {
    let oid = git2::Oid::from_str(&oid_str)?;
    git_repo.find_commit(oid)?
  } else {
    git_repo
      .head()
      .with_context(|| anyhow!("This Rust project is not managed by git"))?
      .peel_to_commit()?
  };
  let tree = commit.tree()?;
  let git_path_prefix =
    git::get_diff_between_repo_dir_and_manifest_file(dir, git_repo_dep, manifest_dep);

  for member_id in metadata.workspace_members.iter() {
    let package_info = metadata
      .packages
      .iter()
      .find(|p| &p.id == member_id)
      .unwrap();
    let targets = &package_info.targets;
    for target in targets.iter() {
      // libのみを対象とする
      // 以下参照
      // - <https://doc.rust-lang.org/reference/linkage.html>
      // - <https://qiita.com/etoilevi/items/4bd4c5b726e41f5a6689>
      let crate_types = &target.crate_types;
      if crate_types.contains(&cargo_metadata::CrateType::Lib)
        || crate_types.contains(&cargo_metadata::CrateType::StaticLib)
        || crate_types.contains(&cargo_metadata::CrateType::RLib)
        || crate_types.contains(&cargo_metadata::CrateType::DyLib)
        || crate_types.contains(&cargo_metadata::CrateType::CDyLib)
      {
        let pwd = target
          .src_path
          .parent()
          .with_context(|| anyhow!("not found directory"))?;
        let content = fs::read_to_string(&target.src_path)?;
        let file = syn::parse_file(&content)?;
        let target_module_info = file::ModuleInfo {
          mod_path: Vec::new(),
          module_file_path: target.src_path.clone().into(),
          items: file.items,
        };
        let mut module_info_list =
          file::get_children_modules(pwd.as_std_path(), &target_module_info)?;
        module_info_list.push(target_module_info);
      }
    }
  }
  Ok(())
}
