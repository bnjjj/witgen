use heck::{ToKebabCase, ToLowerCamelCase, ToSnekCase, ToUpperCamelCase};
use proc_macro2::Ident;

const IDENT_STYLE: &str = "IDENT_STYLE";
const TYPE_STYLE: &str = "TYPE_STYLE";

const LOWER_CAMEL_CASE: &str = "LOWER_CAMEL_CASE";
const UPPER_CAMEL_CASE: &str = "UPPER_CAMEL_CASE";
const SNEK_CASE: &str = "SNEK_CASE";
const SNAKE_CASE: &str = "SNAKE_CASE";
const KEBAB_CASE: &str = "KEBAB_CASE";

enum StringCase {
    LowerCamel,
    Kebab,
    Snake,
    UpperCamel,
}

impl StringCase {
    fn get_from_env(is_type: bool) -> Self {
        let style = if is_type {
            get_type_style_env()
        } else {
            get_ident_style_env()
        };
        match style.as_str() {
            LOWER_CAMEL_CASE => StringCase::LowerCamel,
            SNEK_CASE | SNAKE_CASE => StringCase::Snake,
            UPPER_CAMEL_CASE => StringCase::UpperCamel,
            _ => StringCase::Kebab,
        }
    }

    fn format_str(s: String, is_type: bool) -> String {
        use StringCase::*;
        match StringCase::get_from_env(is_type) {
            LowerCamel => s.to_lower_camel_case(),
            Kebab => s.to_kebab_case(),
            Snake => s.to_snek_case(),
            UpperCamel => s.to_upper_camel_case(),
        }
    }
}

pub(crate) fn get_ident_style_env() -> String {
    std::env::var(IDENT_STYLE).unwrap_or_else(|_| KEBAB_CASE.to_string())
}

pub(crate) fn get_type_style_env() -> String {
    std::env::var(TYPE_STYLE).unwrap_or_else(|_| KEBAB_CASE.to_string())
}

pub fn style_ident(ident: &Ident, is_type: bool) -> String {
    StringCase::format_str(ident.to_string(), is_type)
}
