use std::env;
use std::path::Path;
use std::process::Command;

fn generate_openapi_spec(target_dir: &Path) {
    println!("Generating OpenAPI specification...");
    
    // Get the build profile (debug or release)
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    
    // Run the generate-openapi binary from the correct profile directory
    let binary_path = target_dir.join(&profile).join("generate-openapi");
    
    // Check if the binary exists
    if !binary_path.exists() {
        eprintln!("Warning: generate-openapi binary not found at {}. Run 'cargo build --bin generate-openapi' first.", binary_path.display());
        return;
    }
    
    let exec_result = Command::new(&binary_path)
        .current_dir(".")
        .status();
    
    match exec_result {
        Ok(status) if status.success() => {
            println!("OpenAPI specification generated successfully");
        }
        Ok(status) => {
            eprintln!("Warning: generate-openapi binary failed (exit code: {})", status);
        }
        Err(e) => {
            eprintln!("Warning: Failed to execute generate-openapi binary: {}", e);
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Determine the target platform
    let target = env::var("TARGET").unwrap();

    // === PostgreSQL Setup ===
    build_helpers::postgresql::setup_postgresql(&target);

    // Get the output directory and build profile
    let out_dir = env::var("OUT_DIR").unwrap();
    let _profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // OUT_DIR is typically target/{profile}/build/{package}/out
    // We want to get to target/
    let target_dir = Path::new(&out_dir)
        .parent() // removes /out
        .unwrap()
        .parent() // removes /{package}
        .unwrap()
        .parent() // removes /build
        .unwrap()
        .parent() // removes /{profile}
        .unwrap();

    // === PDFium Binary Download ===
    let pdfium_target_path =
        match build_helpers::pdfium::setup_pdfium(&target, &target_dir, &out_dir) {
            Ok(path) => Some(path),
            Err(e) => {
                eprintln!("Warning: Failed to setup PDFium: {}", e);
                None
            }
        };

    // === Pandoc Binary Download ===
    if let Err(e) = build_helpers::pandoc::setup_pandoc(&target, &target_dir, &out_dir) {
        eprintln!("Warning: Failed to setup Pandoc: {}", e);
    }

    // === Build mistralrs-server ===
    println!("cargo:rerun-if-changed=src-engines/mistralrs-server");
    let mistralrs_source = Path::new("../src-engines/mistralrs-server")
        .canonicalize()
        .ok();
    let _mistralrs_path = match build_helpers::mistralrs::build(
        &target_dir,
        &target,
        mistralrs_source.as_deref(),
    ) {
        Ok(path) => Some(path),
        Err(e) => {
            eprintln!("Warning: Failed to build mistralrs-server: {}", e);
            eprintln!("Continuing without mistralrs-server binary");
            None
        }
    };

    // === Build llama.cpp server ===
    println!("cargo:rerun-if-changed=src-engines/llama.cpp");
    let llamacpp_source = Path::new("../src-engines/llama.cpp").canonicalize().ok();
    let _llamacpp_path = build_helpers::llamacpp::build(
        &target_dir,
        &target,
        llamacpp_source.as_deref(),
    )
    .expect("Failed to build llama.cpp with comprehensive features - build cannot continue");

    // === Set PDFium environment variables ===
    if let Some(ref path) = pdfium_target_path {
        let pdfium_dir = target_dir.join("pdfium");
        build_helpers::pdfium::setup_pdfium_env(&target, path, &pdfium_dir);
    }

    // Also run the default Tauri build script
    tauri_build::build();

    // === Generate OpenAPI specification ===
    println!("cargo:rerun-if-changed=src/route");
    println!("cargo:rerun-if-changed=src/api");
    generate_openapi_spec(&target_dir);
}
