use syn::visit_mut::VisitMut;
use syn::{parse_quote, ImplItemMethod, ItemImpl, Meta};

pub struct ImplVisitor;

impl ImplVisitor {
    pub fn path_attrs_to_docs(impl_: &mut ItemImpl) {
        let mut visitor = ImplVisitor {};
        visitor.visit_item_impl_mut(impl_);
    }
}

impl VisitMut for ImplVisitor {
    fn visit_impl_item_method_mut(&mut self, method: &mut ImplItemMethod) {
        method.attrs.iter_mut().for_each(|attr| {
            println!("{:#?}", attr);
            match attr.parse_meta() {
                Ok(Meta::Path(path)) if path.get_ident().is_some() => {
                    // let ident = path.get_ident().unwrap();
                    println!("is path? {:#?}", attr);
                    let ident = path.get_ident().unwrap().to_string();
                    // let ident = quote::format_ident!("\"{ident}\"");
                    println!("is path? {:#?}", ident);
                    // let Attribute { path, tokens, .. } = attr;
                    let docs = parse_quote! {
                      #[doc =  #ident]
                    };
                    *attr = docs;
                    println!("{:#?}", attr);
                }
                _ => (),
            }
        })
    }
}
