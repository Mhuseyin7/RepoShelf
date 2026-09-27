fn main() {
    // libgit2's bundled Windows build uses these platform APIs. Link them
    // explicitly so the CLI and test binaries work with the MSVC toolchain.
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=crypt32");
    }
}
