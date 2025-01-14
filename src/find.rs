use syn::{Attribute, Generics, Ident, Item, Type, Visibility};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConstData {
  attrs: Vec<Attribute>,
  ident: Ident,
  generics: Generics,
  ty: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ItemTypeData {
  Const(ConstData),
}

pub(crate) fn find_types(items: &[Item]) -> Vec<ItemTypeData> {
  let mut v = Vec::new();
  for item in items.iter() {
    match item {
      Item::Const(item_const) => {
        if let Visibility::Public(_) = item_const.vis {
          v.push(ItemTypeData::Const(ConstData {
            attrs: item_const.attrs.clone(),
            ident: item_const.ident.clone(),
            generics: item_const.generics.clone(),
            ty: item_const.ty.clone(),
          }))
        }
      }
      _ => (),
    }
  }
  v
}
