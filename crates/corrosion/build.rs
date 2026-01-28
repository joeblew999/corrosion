fn main() {
    // Link Windows libraries needed by libgit2-sys and crypto dependencies
    // See: https://github.com/rust-lang/git2-rs/issues/552
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=crypt32");
        println!("cargo:rustc-link-lib=winhttp");
        println!("cargo:rustc-link-lib=rpcrt4");
        println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-lib=secur32");
    }
    build_info_build::build_script();
}
