use anyhow::{anyhow, Context};
use clap::Parser;
use std::fs;
use std::path::Path;

mod file;
mod git;
mod metadata;
mod types;

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
  let manifest = metadata::get_manifest_data_from_path(&manifest_file)?;

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

  if manifest.workspace.is_some() {
    return Err(anyhow!("Not supported for workspaces functionality"));
  }
  let lib_file_path = metadata::lib_file_path(&manifest);
  let pwd = manifest_file.parent().unwrap();
  let lib_file_result = std::fs::read_to_string(pwd.join(&lib_file_path));
  if let Ok(lib_file) = lib_file_result {
    let lib_file_path = pwd.join(lib_file_path);
    let pwd = &lib_file_path.parent().unwrap();

    let old_manifest_file =
      git::get_file_contents(&git_path_prefix, Path::new("Cargo.toml"), &tree, &git_repo)?;
    let old_manifest = metadata::get_manifest_data_from_contents(&old_manifest_file)?;
    let old_lib_file_path = metadata::lib_file_path(&old_manifest);
    if let Ok(old_lib_file) = git::get_file_contents(
      &git_path_prefix,
      Path::new(&old_lib_file_path),
      &tree,
      &git_repo,
    ) {
      let new_lib_file = syn::parse_file(&lib_file)?;
      let new_target_module_info = file::ModuleInfo {
        mod_path: Vec::new(),
        items: new_lib_file.items,
      };
      let mut new_module_info_list = file::get_children_modules(&new_target_module_info, &|p| {
        let file_path = file::check_mod_file_exists(pwd, p);
        match file_path {
          Ok(file_path) => {
            let contents = fs::read_to_string(&file_path);
            match contents {
              Ok(contents) => Ok(contents),
              Err(e) => Err(e.into()),
            }
          }
          Err(e) => Err(e),
        }
      })?;
      new_module_info_list.push(new_target_module_info);

      let pwd_old_lib = Path::new(&old_lib_file_path).parent();
      let src_git_path_prefix = match (&git_path_prefix, pwd_old_lib) {
        (Some(git), Some(pwd)) => Some(git.join(pwd)),
        (Some(git), None) => Some(git.clone()),
        (None, Some(pwd)) => Some(pwd.to_path_buf()),
        (None, None) => None,
      };
      let old_lib_file = syn::parse_file(&old_lib_file)?;
      let old_target_module_info = file::ModuleInfo {
        mod_path: Vec::new(),
        items: old_lib_file.items,
      };
      let mut old_module_info_list = file::get_children_modules(&old_target_module_info, &|p| {
        git::get_mod_file(&src_git_path_prefix, p, &tree, &git_repo)
      })?;
      old_module_info_list.push(old_target_module_info);

      for old_module_info in old_module_info_list.iter() {
        let mod_path_str = old_module_info
          .mod_path
          .iter()
          .map(|i| i.to_string())
          .collect::<Vec<_>>()
          .join("/");
        if let Some(new_module_info) = new_module_info_list
          .iter()
          .find(|info| info.mod_path == old_module_info.mod_path)
        {
          let old_item_type_data = types::extract_types(&old_module_info.items);
          let new_item_type_data = types::extract_types(&new_module_info.items);
          for old_data in old_item_type_data.iter() {
            let result = types::determine_compatibility(old_data, &new_item_type_data);
            match result {
              types::ResultDetermineCompatibility::Uncompatible(_new_data) => {
                println!("Uncompatible: {mod_path_str}::({})", old_data.show_name());
              }
              types::ResultDetermineCompatibility::NotFound => {
                println!(
                  "Uncompatible: {mod_path_str}::({}) does not exist",
                  old_data.show_name()
                );
              }
              types::ResultDetermineCompatibility::Ok => {}
            }
          }
        } else {
          println!("Uncompatible: {mod_path_str} module does not exist")
        }
      }
    } else {
      return Ok(());
    }
  } else {
    return Ok(());
  }
  Ok(())
}
