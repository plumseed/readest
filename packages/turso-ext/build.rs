fn main() {
    // Build scripts run for the host, so cfg!(target_os) reports Windows while
    // cross-compiling Android on Windows. Cargo exposes the actual compilation
    // target through CARGO_CFG_TARGET_OS.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-lib=advapi32");
    }
}
