use std::env;
use std::{fs, path::Path};

fn env_truthy(key: &str) -> bool {
    env::var(key)
        .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn ensure_empty_dir(p: &Path) {
    if let Err(e) = fs::create_dir_all(p) {
        eprintln!("Warning: failed to create placeholder dir {}: {e}", p.display());
    } else {
        let _ = fs::write(p.join(".placeholder"), b"");
    }
}

// use std::process::Command;
//
// fn generate_openapi_spec(target_dir: &Path) {
//     println!("Generating OpenAPI specification...");
//
//     // Get the build profile (debug or release)
//     let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
//
//     // Run the generate-openapi binary from the correct profile directory
//     let binary_path = target_dir.join(&profile).join("generate-openapi");
//
//     // Check if the binary exists
//     if !binary_path.exists() {
//         eprintln!("Warning: generate-openapi binary not found at {}. Run 'cargo build --bin generate-openapi' first.", binary_path.display());
//         return;
//     }
//
//     let exec_result = Command::new(&binary_path).current_dir(".").status();
//
//     match exec_result {
//         Ok(status) if status.success() => {
//             println!("OpenAPI specification generated successfully");
//         }
//         Ok(status) => {
//             eprintln!(
//                 "Warning: generate-openapi binary failed (exit code: {})",
//                 status
//             );
//         }
//         Err(e) => {
//             eprintln!("Warning: Failed to execute generate-openapi binary: {}", e);
//         }
//     }
// }
//
// fn generate_typescript_endpoints() {
//     println!("Generating TypeScript endpoint definitions...");
//
//     // Change to the openapi directory to run the TypeScript generation script
//     let openapi_dir = Path::new("../openapi");
//
//     let exec_result = Command::new("npx")
//         .arg("tsx")
//         .arg("generate-endpoints.ts")
//         .current_dir(openapi_dir)
//         .status();
//
//     match exec_result {
//         Ok(status) if status.success() => {
//             println!("TypeScript endpoints generated successfully");
//         }
//         Ok(status) => {
//             eprintln!(
//                 "Warning: TypeScript endpoint generation failed (exit code: {})",
//                 status
//             );
//         }
//         Err(e) => {
//             eprintln!(
//                 "Warning: Failed to execute TypeScript endpoint generation: {}",
//                 e
//             );
//         }
//     }
// }

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Determine the target platform
    let target = env::var("TARGET").unwrap();

    // === PostgreSQL Setup ===
    env::set_var("POSTGRESQL_VERSION", "17.5.0");

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

    // Decide: skip heavy builds on Windows unless explicitly enabled
    let is_windows = target.contains("windows");
    let default_on = !is_windows;

    let want_mistral  = env::var("BUILD_MISTRALRS").map(|_| env_truthy("BUILD_MISTRALRS")).unwrap_or(default_on);
    let want_llama    = env::var("BUILD_LLAMA_CPP").map(|_| env_truthy("BUILD_LLAMA_CPP")).unwrap_or(default_on);
    let want_pgvector = env::var("BUILD_PGVECTOR").map(|_| env_truthy("BUILD_PGVECTOR")).unwrap_or(default_on);
    let want_age      = env::var("BUILD_APACHE_AGE").map(|_| env_truthy("BUILD_APACHE_AGE")).unwrap_or(default_on);


    // === Build mistralrs-server ===
    if want_mistral {
        println!("cargo:rerun-if-changed=src-engines/mistralrs-server");
        let src = Path::new("../src-engines/mistralrs-server").canonicalize().ok();
        if let Err(e) = build_helpers::mistralrs::build(&target_dir, &target, src.as_deref()) {
            eprintln!("Warning: Failed to build mistralrs-server: {e}");
            eprintln!("Continuing without mistralrs-server binary");
        }
    } else {
        println!("cargo:warning=Skipping mistralrs-server (set BUILD_MISTRALRS=1 to enable).");
    }

    // === Build llama.cpp (optional) ===
    if want_llama {
        println!("cargo:rerun-if-changed=src-engines/llama.cpp");
        let src = Path::new("../src-engines/llama.cpp").canonicalize().ok();
        build_helpers::llamacpp::build(&target_dir, &target, src.as_deref())
            .expect("Failed to build llama.cpp - set BUILD_LLAMA_CPP=0 to skip");
    } else {
        println!("cargo:warning=Skipping llama.cpp (set BUILD_LLAMA_CPP=1 to enable).");
    }

    // === Build pgvector (optional) ===
    if want_pgvector {
        println!("cargo:rerun-if-changed=src-databases/pgvector");
        let src = Path::new("../src-databases/pgvector").canonicalize().ok();
        build_helpers::pgvector::build(&target_dir, &target, src.as_deref())
            .expect("Failed to build pgvector - set BUILD_PGVECTOR=0 to skip");
    } else {
        println!("cargo:warning=Skipping pgvector (set BUILD_PGVECTOR=1 to enable).");
    }

    // === Build Apache AGE (optional) ===
    if want_age {
        println!("cargo:rerun-if-changed=src-databases/apache-age");
        let src = Path::new("../src-databases/apache-age").canonicalize().ok();
        build_helpers::apache_age::build(&target_dir, &target, src.as_deref())
            .expect("Failed to build Apache AGE - set BUILD_APACHE_AGE=0 to skip");
    } else {
        println!("cargo:warning=Skipping Apache AGE (set BUILD_APACHE_AGE=1 to enable).");
    }


    // === Placeholder dirs for skipped components (to satisfy bundle resource globs) ===
    if !want_mistral {
        ensure_empty_dir(&target_dir.join("mistralrs-build").join("bin"));
    }
    if !want_llama {
        ensure_empty_dir(&target_dir.join("llamacpp-build").join("bin")); // existing
        ensure_empty_dir(&target_dir.join("llamacpp").join("bin"));       // <-- add this line
    }
    if !want_pgvector {
        ensure_empty_dir(&target_dir.join("pgvector"));
        ensure_empty_dir(&target_dir.join("pgvector").join("sql"));
    }
    if !want_age {
        ensure_empty_dir(&target_dir.join("apache-age"));
        ensure_empty_dir(&target_dir.join("apache-age").join("sql"));
    }

    // === Set PDFium environment variables ===
    if let Some(ref path) = pdfium_target_path {
        let pdfium_dir = target_dir.join("pdfium");
        build_helpers::pdfium::setup_pdfium_env(&target, path, &pdfium_dir);
    }

    // Also run the default Tauri build script
    tauri_build::build();

    // === Generate OpenAPI specification ===
    // println!("cargo:rerun-if-changed=src/route");
    // println!("cargo:rerun-if-changed=src/api");
    // generate_openapi_spec(&target_dir);
    //
    // // === Generate TypeScript endpoint definitions ===
    // println!("cargo:rerun-if-changed=../openapi/generate-endpoints.ts");
    // generate_typescript_endpoints();
}
