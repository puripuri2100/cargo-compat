use anyhow::{anyhow, Context};
use clap::Parser;
use std::fs;
use std::path::Path;

mod file;
mod metadata;

#[derive(Debug, Clone, Parser)]
struct Args {
  /// Path to working directory
  #[clap(short, long)]
  pub dir: Option<String>,
}

fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let dir = args.dir.unwrap_or(".".to_string());
  let dir = Path::new(&dir);
  let manifest_file = metadata::find_manifest_file(dir)?;
  let metadata = metadata::manifest_to_metadata(&manifest_file)?;
  for member_id in metadata.workspace_members.iter() {
    let package_info = metadata
      .packages
      .iter()
      .find(|p| &p.id == member_id)
      .unwrap();
    let targets = &package_info.targets;
    for target in targets.iter() {
      let pwd = target.src_path.parent().with_context(|| anyhow!("not found directory"))?;
      let crate_types = &target.crate_types;
      let content = fs::read_to_string(&target.src_path)?;
      let file = syn::parse_file(&content)?;
    }
  }
  Ok(())
}
