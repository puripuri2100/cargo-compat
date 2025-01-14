use anyhow::{anyhow, Context};
use git2::{Repository, Tree};
use std::path::{Path, PathBuf};

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
) -> PathBuf {
  if repo_dep == manifest_dep {
    PathBuf::new().join(".")
  } else if repo_dep > manifest_dep {
    let dir_path_list = dir.iter().collect::<Vec<_>>();
    let dir_len = dir_path_list.len();
    let ans = &dir_path_list[(dir_len - repo_dep)..(dir_len - manifest_dep)];
    let mut path = PathBuf::new();
    for p in ans.iter() {
      path = path.join(p);
    }
    path
  } else {
    // 事前に処理されるはず
    unreachable!()
  }
}

#[test]
fn check_manifest_to_metadata() {
  assert_eq!(
    Path::new("hoge/fuga"),
    get_diff_between_repo_dir_and_manifest_file(Path::new("hoge2/hoge/fuga/fuga2"), 3, 1)
  )
}

pub(crate) fn get_file_contents(
  git_path_prefix: &Path,
  file_path: &Path,
  tree: Tree,
  repo: &Repository,
) -> anyhow::Result<String> {
  let filename = git_path_prefix.join(file_path);
  let filename = filename
    .to_str()
    .with_context(|| anyhow!("Failed get filename"))?;
  let tree_entry = tree
    .get_name(filename)
    .with_context(|| anyhow!("Failed get tree entry from {}", filename))?;
  let obj = tree_entry.to_object(repo)?;
  let blob = obj
    .as_blob()
    .with_context(|| anyhow!("Failed get blob from {}", filename))?;
  let contents = String::from_utf8(blob.content().to_vec())?;
  Ok(contents)
}
