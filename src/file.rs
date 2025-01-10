use syn::{File, Item};

// 呼び出しているモジュールを見つける
pub(crate) fn get_mod_files(file: &File) -> Vec<String> {
  let mut mod_items = Vec::new();
  for item in file.items.iter() {
    match item {
      Item::Mod(m) => mod_items.push(m),
      _ => (),
    }
  }
  vec![]
}
