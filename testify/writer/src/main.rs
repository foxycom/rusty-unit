use std::fs;
use std::io::Write;
use quote::ToTokens;
use crate::testsuite::TestSuite;
use syn::Item;

mod testsuite;

fn main() {
    let args = std::env::args()
        .skip(1)
        .map(|a| format!("{}", a))
        .take(2)
        .collect::<Vec<_>>();

    let path = args.get(0).unwrap();
    let content = fs::read_to_string(path).unwrap();
    let mut ast = syn::parse_file(&content).expect("Could not parse source file");

    let tests = &args.get(1).unwrap();

    let testsuite: TestSuite = serde_json::from_str(&args.get(1).unwrap()).unwrap();
    let mut items = testsuite.to_items();

    let tests_mod = ast.items.iter_mut().find_map(|i| {
        if let Item::Mod(item_mod) = i {
            if item_mod.ident.to_string() == "rusty_tests" {
                return Some(item_mod);
            }
        }
        None
    });

    if let Some(tests_mod) = tests_mod {
        let (_, ref mut content) = tests_mod.content.as_mut().unwrap();
        content.clear();
        content.append(&mut items);
    } else {
        let tests_mod: Item = syn::parse_quote! {
            #(#items)*
        };

        ast.items.push(tests_mod);
    }

    let token_stream = ast.to_token_stream();
    let code = token_stream.to_string();

    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .unwrap();

    file.write_all(&code.as_bytes());

    println!("Testsuite is: {:?}", testsuite);
}
