use syn::Attribute;
use ethabi::Result;
use std::collections::HashMap;

pub struct FunctionOptions {
    pub signature: String,
    pub alias: String,
}

pub struct ContractOptions {
    pub path: String,
    pub functions: HashMap<String, FunctionOptions>,
}

impl ContractOptions {
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let options = get_options(attrs, "ethabi_contract_options")?;
        let path = get_option(&options, "path")?;
        let functions = get_function_options(attrs)?
            .into_iter()
            .fold(HashMap::new(), |mut map, option| {
                map.entry(option.signature.to_string()).or_insert(option);
                map
            });
        Ok(Self {
            path,
            functions,
        })
    }
}

fn get_function_options(attrs: &[syn::Attribute]) -> Result<Vec<FunctionOptions>> {
    attrs
        .iter()
        .flat_map(syn::Attribute::parse_meta)
        .filter(|meta| meta.path().is_ident("ethabi_function_options"))
        .filter_map(|meta| -> Option<Vec<syn::NestedMeta>> {
            match meta {
                syn::Meta::List(list) => Some(list.nested.into_iter().collect()),
                _ => None,
            }
        })
        .map(|nested_meta| -> Result<FunctionOptions> {
            let signature = get_option(&nested_meta, "signature")?;
            let alias = get_option(&nested_meta, "alias")?;
            Ok(FunctionOptions { signature, alias })
        })
        .collect()
}

fn get_options(attrs: &[syn::Attribute], name: &str) -> Result<Vec<syn::NestedMeta>> {
    let options = attrs.iter().flat_map(syn::Attribute::parse_meta).find(|meta| meta.path().is_ident(name));

    match options {
        Some(syn::Meta::List(list)) => Ok(list.nested.into_iter().collect()),
        _ => Err("Unexpected meta item".into()),
    }
}

fn get_option(options: &[syn::NestedMeta], name: &str) -> Result<String> {
    let item = options
        .iter()
        .flat_map(|nested| match *nested {
            syn::NestedMeta::Meta(ref meta) => Some(meta),
            _ => None,
        })
        .find(|meta| meta.path().is_ident(name))
        .ok_or_else(|| format!("Expected to find option {}", name))?;

    str_value_of_meta_item(item, name)
}

fn str_value_of_meta_item(item: &syn::Meta, name: &str) -> Result<String> {
    if let syn::Meta::NameValue(ref name_value) = *item {
        if let syn::Lit::Str(ref value) = name_value.lit {
            return Ok(value.value());
        }
    }

    Err(format!(r#"`{}` must be in the form `#[{}="something"]`"#, name, name).into())
}
