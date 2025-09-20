use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // === PostgreSQL Setup ===
    env::set_var("POSTGRESQL_VERSION", "17.5.0");
    env::set_var("DATABASE_URL", "postgresql://postgres:password@127.0.0.1:54321/postgres");
    println!("cargo:rustc-env=POSTGRESQL_VERSION={}", "17.5.0");
    println!("cargo:rustc-env=DATABASE_URL=postgresql://postgres:password@127.0.0.1:54321/postgres");

    // Also run the default Tauri build script
    tauri_build::build();
}