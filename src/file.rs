use anyhow::anyhow;
use std::fs;
use std::path::{Path, PathBuf};
use syn::token::Brace;
use syn::{Ident, Item, Visibility};

#[allow(clippy::type_complexity)]
/// 子モジュールの名前を見つけに行く
pub(crate) fn get_children_file_name(items: &[Item]) -> Vec<(Ident, Option<(Brace, Vec<Item>)>)> {
  let mut mod_name_list = Vec::new();
  for item in items.iter() {
    if let Item::Mod(m) = item {
      // pubのみ扱う
      // `pub(crate)`などは後方互換性に影響を与えないので無視する
      if let Visibility::Public(_) = m.vis {
        mod_name_list.push((m.ident.clone(), m.content.clone()))
      }
    }
  }
  mod_name_list
}

/// 子モジュールのファイルを見に行く
pub(crate) fn check_mod_file_exists(
  pwd: &Path,
  mod_name_list: &[Ident],
) -> anyhow::Result<PathBuf> {
  let mut file_path = pwd.to_path_buf();
  for name in mod_name_list.iter() {
    file_path = file_path.join(name.to_string())
  }
  // ed2015でのファイルのパス
  let ed2015_file_path = file_path.join("mod.rs");
  // ed2018でのファイルのパス
  file_path.set_extension("rs");

  if file_path.try_exists()? {
    return Ok(file_path);
  }

  if ed2015_file_path.try_exists()? {
    return Ok(ed2015_file_path);
  }

  Err(anyhow!(
    "not found child file: {}/{}",
    ed2015_file_path.display(),
    file_path.display()
  ))
}

/// ファイルの情報
#[derive(Debug, Clone)]
pub(crate) struct ModuleInfo {
  /// 階層で区切られたmoduleの名前
  pub mod_path: Vec<Ident>,
  /// moduleが書いてあるファイルのpath
  pub module_file_path: PathBuf,
  /// moduleの中身
  pub items: Vec<Item>,
}

/// 階層が下のモジュールの情報を再帰的に取得する
pub(crate) fn get_children_modules(
  pwd: &Path,
  module_info: &ModuleInfo,
) -> anyhow::Result<Vec<ModuleInfo>> {
  let mut v = Vec::new();

  // ファイルに含まれる`mod`から子階層を呼び出す
  let file_name_list = get_children_file_name(&module_info.items);
  for (name, mod_contents_opt) in file_name_list.iter() {
    let mut p = module_info.mod_path.clone();
    p.push(name.clone());
    if let Some((_, contents)) = mod_contents_opt {
      let info = ModuleInfo {
        mod_path: p,
        module_file_path: module_info.module_file_path.clone(),
        items: contents.clone(),
      };
      v.push(info);
      let children = get_children_modules(pwd, module_info)?;
      v.extend(children);
    } else {
      let file_path = check_mod_file_exists(pwd, &p)?;
      let contents = fs::read_to_string(&file_path)?;
      let file = syn::parse_file(&contents)?;
      let info = ModuleInfo {
        mod_path: p,
        module_file_path: file_path,
        items: file.items,
      };
      v.push(info);
      let children = get_children_modules(pwd, module_info)?;
      v.extend(children);
    }
  }
  Ok(v)
}
