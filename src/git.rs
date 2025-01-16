use anyhow::{anyhow, Context};
use git2::{Repository, Tree};
use std::path::{Path, PathBuf};
use syn::Ident;

pub(crate) fn find_repo(dir: &Path) -> anyhow::Result<(usize, Repository)> {
  // 見つからなかったら階層を親側に辿る
  for (i, path) in dir.ancestors().enumerate() {
    if let Ok(repo) = git2::Repository::open(path) {
      return Ok((i, repo));
    }
  }

  Err(anyhow!("Not find git repository for {}", dir.display()))
}

#[allow(clippy::comparison_chain)]
pub(crate) fn get_diff_between_repo_dir_and_manifest_file(
  dir: &Path,
  repo_dep: usize,
  manifest_dep: usize,
) -> Option<PathBuf> {
  if repo_dep == manifest_dep {
    None
  } else if repo_dep > manifest_dep {
    let dir_path_list = dir.iter().collect::<Vec<_>>();
    let dir_len = dir_path_list.len();
    let ans = &dir_path_list[(dir_len - repo_dep)..(dir_len - manifest_dep)];
    let mut path = PathBuf::new();
    for p in ans.iter() {
      path = path.join(p);
    }
    Some(path)
  } else {
    // 事前に処理されるはず
    unreachable!()
  }
}

#[test]
fn check_manifest_to_metadata() {
  assert_eq!(
    Some(Path::new("hoge/fuga").to_path_buf()),
    get_diff_between_repo_dir_and_manifest_file(Path::new("hoge2/hoge/fuga/fuga2"), 3, 1)
  )
}

pub(crate) fn get_file_contents(
  git_path_prefix: &Option<PathBuf>,
  file_path: &Path,
  tree: &Tree,
  repo: &Repository,
) -> anyhow::Result<String> {
  let filename = if let Some(git_path_prefix) = git_path_prefix {
    git_path_prefix.join(file_path)
  } else {
    file_path.to_path_buf()
  };
  let tree_entry = tree
    .get_path(&filename)
    .with_context(|| anyhow!("Failed get tree entry from {}", filename.display()))?;
  let obj = tree_entry.to_object(repo)?;
  let blob = obj
    .as_blob()
    .with_context(|| anyhow!("Failed get blob from {}", filename.display()))?;
  let contents = String::from_utf8(blob.content().to_vec())?;
  Ok(contents)
}

pub(crate) fn get_mod_file(
  git_path_prefix: &Option<PathBuf>,
  p: &[Ident],
  tree: &Tree,
  repo: &Repository,
) -> anyhow::Result<String> {
  let p = if p.is_empty() {
    "lib".to_string()
  } else {
    p.iter()
      .map(|i| i.to_string())
      .collect::<Vec<_>>()
      .join("/")
  };
  let file_name_e2015 = Path::new(&p).join("mod.rs");
  let mut file_name_e2018 = Path::new(&p).to_path_buf();
  file_name_e2018.set_extension("rs");
  if let Ok(file) = get_file_contents(git_path_prefix, &file_name_e2015, tree, repo) {
    Ok(file)
  } else if let Ok(file) = get_file_contents(git_path_prefix, &file_name_e2018, tree, repo) {
    Ok(file)
  } else {
    Err(anyhow!(
      "Failed get module entry file: {} or {}",
      file_name_e2015.display(),
      file_name_e2018.display()
    ))
  }
}
