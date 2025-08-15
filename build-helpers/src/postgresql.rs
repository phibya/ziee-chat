use std::env;

pub fn setup_postgresql(target: &str) {
    // Set PostgreSQL version for postgresql_embedded crate
    env::set_var("POSTGRESQL_VERSION", "17.5.0");
    println!("cargo:rustc-env=POSTGRESQL_VERSION=17.5.0");
    println!("Setting PostgreSQL version to 17.5.0");
    
    // Force x86_64 PostgreSQL binaries on Windows aarch64
    if target == "aarch64-pc-windows-msvc" {
        // Force specific PostgreSQL release URL for Windows aarch64 to use x86_64 binary
        let postgresql_url = "https://github.com/theseus-rs/postgresql-binaries/releases/download/17.5.0/postgresql-17.5.0-x86_64-pc-windows-msvc.zip";
        env::set_var("POSTGRESQL_RELEASES_URL", postgresql_url);
        println!("cargo:rustc-env=POSTGRESQL_RELEASES_URL={}", postgresql_url);
        
        // Set environment variables to force x86_64 PostgreSQL binary download
        println!("cargo:rustc-env=POSTGRESQL_ARCH=x86_64-pc-windows-msvc");
        println!("cargo:rustc-env=TARGET_ARCH_OVERRIDE=x86_64-pc-windows-msvc");
        
        // Also set it for the postgresql_embedded crate at build time
        env::set_var("POSTGRESQL_ARCH", "x86_64-pc-windows-msvc");
        env::set_var("TARGET_ARCH_OVERRIDE", "x86_64-pc-windows-msvc");
        
        println!("Forcing x86_64 PostgreSQL binaries for Windows aarch64 target");
        println!("Setting PostgreSQL releases URL to: {}", postgresql_url);
    }
}