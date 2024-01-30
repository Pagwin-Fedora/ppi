pub fn main() {
    println!("cargo:rustc-link-lib=git2");
    println!("cargo:rustc-link-lib=ssh2");
}
