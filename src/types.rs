use syn::{
  Attribute, Fields, FnArg, Generics, Ident, Item, ReturnType, StaticMutability, Type, Variant,
  Visibility,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConstData {
  pub attrs: Vec<Attribute>,
  pub ident: Ident,
  pub generics: Generics,
  pub ty: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaticData {
  pub attrs: Vec<Attribute>,
  pub is_mut: bool,
  pub ident: Ident,
  pub ty: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnionData {
  pub attrs: Vec<Attribute>,
  pub ident: Ident,
  pub generics: Generics,
  pub fields: Fields,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TypeData {
  pub attrs: Vec<Attribute>,
  pub ident: Ident,
  pub generics: Generics,
  pub ty: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StructData {
  pub attrs: Vec<Attribute>,
  pub ident: Ident,
  pub generics: Generics,
  pub fields: Fields,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnumData {
  pub attrs: Vec<Attribute>,
  pub ident: Ident,
  pub generics: Generics,
  pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FnData {
  pub attrs: Vec<Attribute>,
  pub ident: Ident,
  pub is_const: bool,
  pub is_async: bool,
  pub is_unsafe: bool,
  pub generics: Generics,
  pub args: Vec<FnArg>,
  pub return_type: ReturnType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MacroData {
  pub attrs: Vec<Attribute>,
  pub ident: Ident,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ItemTypeData {
  Const(ConstData),
  Static(StaticData),
  Union(UnionData),
  Type(TypeData),
  Struct(StructData),
  Enum(EnumData),
  Fn(FnData),
  Macro(MacroData),
}

impl ItemTypeData {
  pub(crate) fn show_name(&self) -> String {
    match self {
      Self::Const(d) => format!("const {}", d.ident),
      Self::Static(d) => format!("static {}", d.ident),
      Self::Union(d) => format!("union {}", d.ident),
      Self::Type(d) => format!("type {}", d.ident),
      Self::Struct(d) => format!("struct {}", d.ident),
      Self::Enum(d) => format!("enum {}", d.ident),
      Self::Fn(d) => format!("fn {}", d.ident),
      Self::Macro(d) => format!("macro {}", d.ident),
    }
  }
}

/// moduleの中から型情報を抽出する関数
pub(crate) fn extract_types(items: &[Item]) -> Vec<ItemTypeData> {
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
      Item::Static(item_static) => {
        if let Visibility::Public(_) = item_static.vis {
          v.push(ItemTypeData::Static(StaticData {
            attrs: item_static.attrs.clone(),
            is_mut: item_static.mutability != StaticMutability::None,
            ident: item_static.ident.clone(),
            ty: item_static.ty.clone(),
          }))
        }
      }
      Item::Union(item_union) => {
        if let Visibility::Public(_) = item_union.vis {
          v.push(ItemTypeData::Union(UnionData {
            attrs: item_union.attrs.clone(),
            ident: item_union.ident.clone(),
            generics: item_union.generics.clone(),
            fields: Fields::Named(item_union.fields.clone()),
          }))
        }
      }
      Item::Type(item_type) => {
        if let Visibility::Public(_) = item_type.vis {
          v.push(ItemTypeData::Type(TypeData {
            attrs: item_type.attrs.clone(),
            ident: item_type.ident.clone(),
            generics: item_type.generics.clone(),
            ty: item_type.ty.clone(),
          }))
        }
      }
      Item::Struct(item_struct) => {
        if let Visibility::Public(_) = item_struct.vis {
          v.push(ItemTypeData::Struct(StructData {
            attrs: item_struct.attrs.clone(),
            ident: item_struct.ident.clone(),
            generics: item_struct.generics.clone(),
            fields: item_struct.fields.clone(),
          }))
        }
      }
      Item::Enum(item_enum) => {
        if let Visibility::Public(_) = item_enum.vis {
          let mut variants = Vec::new();
          for v in item_enum.variants.iter() {
            variants.push(v.clone());
          }
          v.push(ItemTypeData::Enum(EnumData {
            attrs: item_enum.attrs.clone(),
            ident: item_enum.ident.clone(),
            generics: item_enum.generics.clone(),
            variants,
          }))
        }
      }
      Item::Fn(item_fn) => {
        if let Visibility::Public(_) = item_fn.vis {
          let sig = &item_fn.sig;
          let is_const = sig.constness.is_some();
          let is_async = sig.asyncness.is_some();
          let is_unsafe = sig.unsafety.is_some();
          let mut args = Vec::new();
          for arg in sig.inputs.iter() {
            args.push(arg.clone())
          }
          v.push(ItemTypeData::Fn(FnData {
            attrs: item_fn.attrs.clone(),
            ident: sig.ident.clone(),
            is_const,
            is_async,
            is_unsafe,
            generics: sig.generics.clone(),
            args,
            return_type: sig.output.clone(),
          }))
        }
      }
      Item::Macro(item_macro) => {
        if let Some(name) = &item_macro.ident {
          v.push(ItemTypeData::Macro(MacroData {
            attrs: item_macro.attrs.clone(),
            ident: name.clone(),
          }))
        }
      }
      _ => (),
    }
  }
  v
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResultDetermineCompatibility {
  Ok,
  Uncompatible(ItemTypeData),
  NotFound,
}

fn fileds_eq(old_fields: &Fields, new_fields: &Fields) -> bool {
  match (old_fields, new_fields) {
    (Fields::Unit, Fields::Unit) => true,
    (Fields::Unnamed(old_unnamed), Fields::Unnamed(new_unnamed)) => {
      let old_len = old_unnamed.unnamed.len();
      let new_len = new_unnamed.unnamed.len();
      if old_len == new_len {
        let f_zip = old_unnamed.unnamed.iter().zip(new_unnamed.unnamed.iter());
        for (old_f, new_f) in f_zip {
          if old_f.ty != new_f.ty {
            return false;
          }
        }
        true
      } else {
        false
      }
    }
    (Fields::Named(old_named), Fields::Named(new_named)) => {
      for old_f in old_named.named.iter() {
        if let Visibility::Public(_) = old_f.vis {
          if let Some(old_ident) = &old_f.ident {
            if let Some(new_f) = new_named
              .named
              .iter()
              .find(|f| f.ident.clone().map(|i| i.to_string()) == Some(old_ident.to_string()))
            {
              if old_f.ty != new_f.ty {
                return false;
              }
            } else {
              return false;
            }
          }
        }
      }
      true
    }
    _ => false,
  }
}

/// 型を探して互換性の有無を判定する
///
/// TODO:
/// - attributes
/// - generics
/// - impl
/// - trait
pub(crate) fn determine_compatibility(
  old_data: &ItemTypeData,
  new_data_list: &[ItemTypeData],
) -> ResultDetermineCompatibility {
  for new_data in new_data_list.iter() {
    match (old_data, new_data) {
      (ItemTypeData::Const(old_const), ItemTypeData::Const(new_const)) => {
        if old_const.ident == new_const.ident {
          if *old_const.ty == *new_const.ty {
            return ResultDetermineCompatibility::Ok;
          } else {
            return ResultDetermineCompatibility::Uncompatible(ItemTypeData::Const(
              new_const.clone(),
            ));
          }
        }
      }
      (ItemTypeData::Static(old_static), ItemTypeData::Static(new_static)) => {
        if old_static.ident == new_static.ident {
          if *old_static.ty == *new_static.ty && old_static.is_mut == new_static.is_mut {
            return ResultDetermineCompatibility::Ok;
          } else {
            return ResultDetermineCompatibility::Uncompatible(ItemTypeData::Static(
              new_static.clone(),
            ));
          }
        }
      }
      (ItemTypeData::Union(old_union), ItemTypeData::Union(new_union)) => {
        if old_union.ident == new_union.ident {
          if fileds_eq(&old_union.fields, &new_union.fields) {
            return ResultDetermineCompatibility::Ok;
          } else {
            return ResultDetermineCompatibility::Uncompatible(ItemTypeData::Union(
              new_union.clone(),
            ));
          }
        }
      }
      (ItemTypeData::Type(old_type), ItemTypeData::Type(new_type)) => {
        if old_type.ident == new_type.ident {
          if *old_type.ty == *new_type.ty {
            return ResultDetermineCompatibility::Ok;
          } else {
            return ResultDetermineCompatibility::Uncompatible(ItemTypeData::Type(
              new_type.clone(),
            ));
          }
        }
      }
      (ItemTypeData::Struct(old_struct), ItemTypeData::Struct(new_struct)) => {
        if old_struct.ident == new_struct.ident {
          if fileds_eq(&old_struct.fields, &new_struct.fields) {
            return ResultDetermineCompatibility::Ok;
          } else {
            return ResultDetermineCompatibility::Uncompatible(ItemTypeData::Struct(
              new_struct.clone(),
            ));
          }
        }
      }
      (ItemTypeData::Enum(old_enum), ItemTypeData::Enum(new_enum)) => {
        if old_enum.ident == new_enum.ident {
          for old_v in old_enum.variants.iter() {
            if let Some(new_v) = new_enum
              .variants
              .iter()
              .find(|v| v.ident.clone() == old_v.ident)
            {
              if !fileds_eq(&old_v.fields, &new_v.fields) {
                return ResultDetermineCompatibility::Uncompatible(ItemTypeData::Enum(
                  new_enum.clone(),
                ));
              }
            } else {
              // not found fields
              return ResultDetermineCompatibility::Uncompatible(ItemTypeData::Enum(
                new_enum.clone(),
              ));
            }
          }
          return ResultDetermineCompatibility::Ok;
        }
      }
      (ItemTypeData::Fn(old_fn), ItemTypeData::Fn(new_fn)) => {
        if old_fn.ident == new_fn.ident {
          let return_ty_eq = match (&old_fn.return_type, &new_fn.return_type) {
            (ReturnType::Default, ReturnType::Default) => true,
            (ReturnType::Type(_, old_re_ty), ReturnType::Type(_, new_re_ty)) => {
              *old_re_ty == *new_re_ty
            }
            _ => false,
          };
          let old_args_len = old_fn.args.len();
          let new_args_len = new_fn.args.len();
          let args_eq = if old_args_len == new_args_len {
            let args_zip = old_fn.args.iter().zip(new_fn.args.iter());
            let mut is_eq = true;
            for (old_arg, new_arg) in args_zip {
              match (old_arg, new_arg) {
                (FnArg::Receiver(old_receiver), FnArg::Receiver(new_receiver)) => {
                  is_eq = old_receiver.reference.is_some() == new_receiver.reference.is_some()
                    && old_receiver.mutability.is_some() == new_receiver.mutability.is_some()
                    && old_receiver.ty == new_receiver.ty;
                }
                (FnArg::Typed(old_typed), FnArg::Typed(new_typed)) => {
                  is_eq = old_typed.pat == new_typed.pat && old_typed.ty == new_typed.ty;
                }
                _ => is_eq = false,
              }
            }
            is_eq
          } else {
            false
          };
          if (old_fn.is_async == new_fn.is_async)
            && (old_fn.is_const == new_fn.is_const)
            && return_ty_eq
            && args_eq
          {
            return ResultDetermineCompatibility::Ok;
          } else {
            return ResultDetermineCompatibility::Uncompatible(ItemTypeData::Fn(new_fn.clone()));
          }
        }
      }
      (ItemTypeData::Macro(old_macro), ItemTypeData::Macro(new_macro)) => {
        if old_macro.ident == new_macro.ident {
          return ResultDetermineCompatibility::Ok;
        }
      }
      _ => (),
    }
  }
  ResultDetermineCompatibility::NotFound
}
