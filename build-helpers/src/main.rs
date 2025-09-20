use std::env;
use std::path::{Path, PathBuf};

fn strip_unc_prefix(path: PathBuf) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.starts_with(r"\\?\") {
        PathBuf::from(&path_str[4..]) // Remove \\?\ prefix
    } else {
        path
    }
}

fn copy_llamacpp_to_target(build_target_dir: &Path, src_tauri_target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source_llamacpp_dir = build_target_dir.join("llamacpp");
    let dest_llamacpp_dir = src_tauri_target_dir.join("llamacpp");

    if !source_llamacpp_dir.exists() {
        return Err(format!("Source llamacpp directory not found: {}", source_llamacpp_dir.display()).into());
    }

    // Create destination directory
    std::fs::create_dir_all(&dest_llamacpp_dir)?;

    // Copy the entire llamacpp directory
    copy_dir_recursive(&source_llamacpp_dir, &dest_llamacpp_dir)?;

    Ok(())
}

fn copy_mistralrs_to_target(build_target_dir: &Path, src_tauri_target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let source_mistralrs_build_dir = build_target_dir.join("mistralrs-build");
    let source_bin_dir = source_mistralrs_build_dir.join("bin");
    let dest_mistralrs_dir = src_tauri_target_dir.join("mistralrs");
    let dest_bin_dir = dest_mistralrs_dir.join("bin");

    if !source_bin_dir.exists() {
        return Err(format!("Source mistralrs bin directory not found: {}", source_bin_dir.display()).into());
    }

    // Create destination directories
    std::fs::create_dir_all(&dest_bin_dir)?;

    // Copy the bin directory contents
    copy_dir_recursive(&source_bin_dir, &dest_bin_dir)?;

    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn build_component(component: &str, src_tauri_target_dir: &Path, build_target_dir: &Path, target: &str) -> Result<(), Box<dyn std::error::Error>> {
    match component {
        "pgvector" => {
            let pgvector_source = Path::new("../src-databases/pgvector").canonicalize()
                .or_else(|_| std::env::current_dir().unwrap().parent().unwrap().join("src-databases/pgvector").canonicalize())
                .map(strip_unc_prefix).ok();

            match build_helpers::pgvector::build(&src_tauri_target_dir, &target, pgvector_source.as_deref()) {
                Ok(path) => {
                    println!("pgvector built successfully at: {}", path.display());
                    Ok(())
                }
                Err(e) => Err(e)
            }
        }
        "apache_age" => {
            let apache_age_source = Path::new("../src-databases/apache-age").canonicalize()
                .or_else(|_| std::env::current_dir().unwrap().parent().unwrap().join("src-databases/apache-age").canonicalize())
                .map(strip_unc_prefix).ok();

            match build_helpers::apache_age::build(&src_tauri_target_dir, &target, apache_age_source.as_deref()) {
                Ok(path) => {
                    println!("Apache AGE built successfully at: {}", path.display());
                    Ok(())
                }
                Err(e) => Err(e)
            }
        }
        "pandoc" => {
            match build_helpers::pandoc::setup_pandoc(&target, &src_tauri_target_dir, &std::env::var("OUT_DIR").unwrap()) {
                Ok(()) => {
                    println!("Pandoc setup completed successfully");
                    Ok(())
                }
                Err(e) => Err(e)
            }
        }
        "pdfium" => {
            match build_helpers::pdfium::setup_pdfium(&target, &src_tauri_target_dir, &std::env::var("OUT_DIR").unwrap()) {
                Ok(path) => {
                    println!("PDFium setup completed successfully at: {}", path.display());
                    Ok(())
                }
                Err(e) => Err(e)
            }
        }
        "mistralrs" => {
            let mistralrs_source = Path::new("../src-engines/mistralrs-server").canonicalize()
                .or_else(|_| std::env::current_dir().unwrap().parent().unwrap().join("src-engines/mistralrs-server").canonicalize())
                .map(strip_unc_prefix).ok();

            match build_helpers::mistralrs::build(&build_target_dir, &target, mistralrs_source.as_deref()) {
                Ok(path) => {
                    println!("mistralrs-server built successfully at: {}", path.display());

                    // Copy to final location
                    copy_mistralrs_to_target(&build_target_dir, &src_tauri_target_dir)?;
                    println!("mistralrs-server copied to final location");
                    Ok(())
                }
                Err(e) => Err(e)
            }
        }
        "llamacpp" => {
            let llamacpp_source = Path::new("../src-engines/llama.cpp").canonicalize()
                .or_else(|_| std::env::current_dir().unwrap().parent().unwrap().join("src-engines/llama.cpp").canonicalize())
                .map(strip_unc_prefix).ok();

            match build_helpers::llamacpp::build(&build_target_dir, &target, llamacpp_source.as_deref()) {
                Ok(path) => {
                    println!("llama.cpp server built successfully at: {}", path.display());

                    // Copy to final location
                    copy_llamacpp_to_target(&build_target_dir, &src_tauri_target_dir)?;
                    println!("llama.cpp server copied to final location");
                    Ok(())
                }
                Err(e) => Err(e)
            }
        }
        "postgres" => {
            println!("Setting up PostgreSQL server...");

            // Use tokio runtime to run the async function
            let rt = tokio::runtime::Runtime::new().map_err(|e| format!("Failed to create tokio runtime: {}", e))?;
            rt.block_on(async {
                build_helpers::postgres::setup_postgres().await
            }).map_err(|e| format!("PostgreSQL setup failed: {}", e))?;

            println!("PostgreSQL server setup successfully");
            Ok(())
        }
        _ => Err(format!("Unknown component: {}", component).into())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: build_helper <component|all>");
        eprintln!("Components: pgvector, apache_age, pandoc, pdfium, mistralrs, llamacpp, postgres");
        eprintln!("Use 'all' to build all components including postgres setup");
        std::process::exit(1);
    }

    let component = &args[1];

    // For final output, use src-tauri/target, but for build artifacts use build-helpers/target
    let src_tauri_target_dir = {
        let canonical = Path::new("../src-tauri/target").canonicalize()
            .unwrap_or_else(|_| std::env::current_dir().unwrap().parent().unwrap().join("src-tauri/target"));
        strip_unc_prefix(canonical)
    };

    let build_target_dir = {
        let canonical = Path::new("target").canonicalize()
            .unwrap_or_else(|_| std::env::current_dir().unwrap().join("target"));
        strip_unc_prefix(canonical)
    };
    let target = env::var("TARGET").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "x86_64-pc-windows-msvc".to_string()
        } else if cfg!(target_os = "macos") {
            "x86_64-apple-darwin".to_string()
        } else {
            "x86_64-unknown-linux-gnu".to_string()
        }
    });

    // Set OUT_DIR environment variable like build.rs does - use temp directory
    let out_dir = std::env::temp_dir().join("build-helpers-out");
    std::fs::create_dir_all(&out_dir)?;
    env::set_var("OUT_DIR", &out_dir);
    println!("Set OUT_DIR to: {}", out_dir.display());

    println!("Building {} for target {}", component, target);
    println!("Build artifacts will be in: {}", build_target_dir.display());
    println!("Final output will be in: {}", src_tauri_target_dir.display());
    println!("Current working directory: {:?}", std::env::current_dir());
    println!("Build target directory exists: {}", build_target_dir.exists());
    println!("Src-tauri target directory exists: {}", src_tauri_target_dir.exists());

    match component.as_str() {
        "all" => {
            println!("Building all components...");
            let components = ["pgvector", "apache_age", "pandoc", "pdfium", "mistralrs", "llamacpp", "postgres"];

            for comp in &components {
                println!("\n=== Building {} ===", comp);
                if let Err(e) = build_component(comp, &src_tauri_target_dir, &build_target_dir, &target) {
                    eprintln!("Failed to build {}: {}", comp, e);
                    std::process::exit(1);
                }
                println!("✓ {} completed successfully", comp);
            }

            println!("\n🎉 All components built successfully!");
        }
        comp => {
            println!("Building {}...", comp);
            if let Err(e) = build_component(comp, &src_tauri_target_dir, &build_target_dir, &target) {
                eprintln!("Failed to build {}: {}", comp, e);
                std::process::exit(1);
            }
            println!("✓ {} completed successfully", comp);
        }
    }

    Ok(())
}