use std::env::var;
use std::path::Path;

fn main() {
    if var("TARGET")
        .map(|target| target == "arm-linux-androideabi")
        .unwrap_or(false)
    {
        let dir = var("CARGO_MANIFEST_DIR").unwrap();
        println!(
            "cargo:rustc-link-search=native={}",
            Path::new(&dir).join("lib").display()
        );
    }
}
