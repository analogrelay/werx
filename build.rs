use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=data/namedata.json");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let json_path = Path::new(&manifest_dir).join("data/namedata.json");
    let out_dir = Path::new(&manifest_dir).join("src");
    let dest_path = out_dir.join("namedata.rs");

    let json_content = fs::read_to_string(&json_path).expect("Failed to read data/namedata.json");

    let data: serde_json::Value =
        serde_json::from_str(&json_content).expect("Failed to parse data/namedata.json");

    let adjectives = data["adjectives"]
        .as_array()
        .expect("adjectives should be an array");
    let nouns = data["nouns"].as_array().expect("nouns should be an array");

    let mut output = String::new();
    output.push_str("//! Auto-generated from data/namedata.json by build.rs\n");
    output.push_str("//! Do not edit manually.\n\n");

    // Generate adjectives array
    output.push_str("pub const ADJECTIVES: &[&str] = &[\n");
    for adj in adjectives {
        let s = adj.as_str().expect("adjective should be a string");
        output.push_str(&format!("    \"{}\",\n", s));
    }
    output.push_str("];\n\n");

    // Generate nouns array
    output.push_str("pub const NOUNS: &[&str] = &[\n");
    for noun in nouns {
        let s = noun.as_str().expect("noun should be a string");
        output.push_str(&format!("    \"{}\",\n", s));
    }
    output.push_str("];\n");

    fs::write(&dest_path, output).expect("Failed to write src/namedata.rs");
}
