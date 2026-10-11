use syn::{Data, Expr, Field, Fields, Lit};

pub fn get_doc_comment(field: &syn::Field) -> Option<String> {
    for attr in &field.attrs {
        if attr.path().is_ident("doc")
            && let Ok(meta) = attr.meta.require_name_value()
            && let Expr::Lit(expr_lit) = &meta.value
            && let Lit::Str(lit) = &expr_lit.lit
        {
            return Some(lit.value());
        }
    }
    None
}

pub fn field_to_config_name(field: &Field) -> String {
    field
        .ident
        .clone()
        .unwrap()
        .to_string()
        .trim_start_matches("r#")
        .replace('_', "-")
}

pub fn get_fields(data: &Data) -> Vec<&Field> {
    match data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => fields_named.named.iter().collect::<Vec<&Field>>(),
            Fields::Unnamed(fields_unnamed) => {
                fields_unnamed.unnamed.iter().collect::<Vec<&Field>>()
            }
            Fields::Unit => Vec::new(),
        },
        _ => {
            panic!("Docs derive only works for structs");
        }
    }
}
