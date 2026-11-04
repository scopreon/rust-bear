fn main() {
    let library_name = std::env::var("LIBRARY_NAME").unwrap();
    println!("cargo::rustc-link-search=/workspace/rust-bear");
    println!("cargo::rustc-link-arg=-l:{}.a", library_name);
}
