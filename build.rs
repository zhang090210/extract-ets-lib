// build.rs
fn main() {
    println!("cargo:rustc-link-search=native=C:\\Program Files\\wkhtmltopdf\\lib");
    println!("cargo:rustc-link-lib=static=wkhtmltox");
}