fn main() {
    println!("cargo:rustc-link-search={}", env!("TREE_SITTER_BASH"));
}
