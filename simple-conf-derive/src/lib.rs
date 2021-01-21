use itertools::Itertools;
use proc_macro_error::abort_call_site;
use syn::{Attribute, Data, Field, Fields, Ident, Lit, Meta, MetaNameValue, NestedMeta, parse_macro_input, Token, DeriveInput};

use crate::ConfigInputType::{Path, Serialized};
use proc_macro::TokenStream;
use quote::quote;

enum ConfigInputType {
    Path(Lit),
    Serialized(Lit)
}

struct ConfigAttributes {
    input: ConfigInputType,
    serializer: Option<Lit>,
    deserializer: Option<Lit>,
    is_structopt_present: bool
}

struct ConfigField {
    name: Ident,
    save: Option<Lit>
}

impl ConfigAttributes {
    fn new(attributes: &Vec<Attribute>) -> Self {
        let name_values = parse_attribute(attributes, "from_config");
        if let Some((path, serialized, serializer, deserializer)) = parse_meta_to_lit(
            name_values,
            vec!("path", "serialized", "serializer", "deserializer")
        ).into_iter().tuples().next() {
            let input = if let Some(lit) = path {
                Path(lit)
            } else if let Some(lit) = serialized {
                Serialized(lit)
            } else {
                abort_call_site!("Expected either path or serialized argument.");
            };

            let struct_opt = parse_attribute(attributes, "StructOpt");
            return ConfigAttributes {
                input,
                serializer,
                deserializer,
                is_structopt_present: if struct_opt.is_empty() { false } else { true }
            };
        }
        unreachable!();
    }
}

impl ConfigField {
    fn new(field: Field) -> Self {
        let attribute = parse_attribute(&field.attrs, "from_config");
        let save = parse_meta_to_lit(
            attribute,
            vec!("save")
        ).remove(0);
        ConfigField {
            name: field.ident.expect("Only named fields are supported."),
            save
        }
    }
}

#[proc_macro_derive(SimpleConf, attributes(from_config))]
pub fn generate_options(input: TokenStream) -> TokenStream {
    let derived_input = parse_macro_input!(input as DeriveInput);

    let config_attributes= ConfigAttributes::new(&derived_input.attrs);
    let fields = parse_fields(derived_input.data);
    let struct_name = derived_input.ident;

    let serde_struct = quote! {
        struct __SimpleConfSerdeInternal {

        }
    };
    let implementation = quote! {
        impl ::simple_conf::SimpleConf for #struct_name {
            fn from_serialized(serialized: &str) -> Self {

            }

            fn from_path(path: &Path) -> Self {

            }

            fn to_serialized(&self) -> &str {

            }

            fn to_path(&self, path: &Path) {

            }
        }
    };
}

fn parse_fields(data: Data) -> Vec<ConfigField> {
    match data {
        Data::Struct(data) => {
            match data.fields {
                Fields::Named(fields) => fields.named,
                _ => abort_call_site!("Only named fields are supported.")
            }
        }
        _ => abort_call_site!("Only structs are supported.")
    }.into_iter().map(|field| ConfigField::new(field)).collect()
}

fn parse_attribute(input: &Vec<Attribute>, attribute: &str) -> Vec<MetaNameValue> {
    input
        .iter()
        .filter(|attr| attr.path.is_ident(attribute))
        .flat_map(|attr| match attr.parse_meta() {
            Ok(meta) => match meta {
                Meta::List(list) => list.nested,
                _ => abort_call_site!("Expected MetaList")
            }
            _ => abort_call_site!("Expected Meta")
        })
        .map(|nested_meta| match nested_meta {
            NestedMeta::Meta(meta) => match meta {
                Meta::NameValue(name_value) => name_value,
                _ => abort_call_site!("Expected NameValue Meta")
            }
            _ => abort_call_site!("Expected Meta")
        })
        .collect()
}

fn parse_meta_to_lit(name_values: Vec<MetaNameValue>, search_for: Vec<&str>) -> Vec<Option<Lit>> {
    if name_values.len() > search_for.len() {
        abort_call_site!("Too many arguments.");
    }

    let mut lit_options: Vec<Option<Lit>> = vec![None; search_for.len()];
    for value in name_values {
        let value_name = &*value.path.get_ident().expect("Expected Path as Ident.").to_string();
        let position = search_for.iter().position(|&e| e.eq(value_name)).expect("Unsupported argument.");
        lit_options.get_mut(position).unwrap().replace(value.lit);
    }
    lit_options
}

