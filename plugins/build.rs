use std::collections::HashMap;

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=messages.ftl");

    let manifest = std::fs::read_to_string("Cargo.toml").expect("failed to read Cargo.toml");
    let table: HashMap<String, toml::Value> = toml::from_str(&manifest).expect("invalid Cargo.toml");

    let metadata = &table["package"]["metadata"]["plugin"];
    let translation = &metadata["translation"];

    let id = metadata["id"].as_str().expect("missing metadata.plugin.id");
    let name = metadata["name"].as_str().expect("missing metadata.plugin.name");
    let lang_name = translation["language_name"].as_str().expect("missing translation.language_name");

    println!("cargo:rustc-env=TRANSLATION_ID={id}");
    println!("cargo:rustc-env=TRANSLATION_DISPLAY_NAME={name}");
    println!("cargo:rustc-env=TRANSLATION_LANG_NAME={lang_name}");
}
