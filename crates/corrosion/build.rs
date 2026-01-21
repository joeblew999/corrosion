fn main() {
    // Link Windows libraries needed by libgit2
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=advapi32");
    }
    build_info_build::build_script();
}
