fn main() {
    // cef-dll-sys places the Linux runtime next to the browser executable.
    println!("cargo:rustc-link-arg-bin=crest-browser=-Wl,-rpath,$ORIGIN");
}
