use std::fs;
use std::path::Path;
use ziee_lib::route::create_rest_router_internal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating OpenAPI specification...");
    
    // Get the OpenAPI spec from the existing route configuration
    let (api, _router) = create_rest_router_internal();
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(&api)?;
    
    // Ensure the openapi directory exists
    let openapi_dir = Path::new("../openapi");
    if !openapi_dir.exists() {
        fs::create_dir_all(openapi_dir)?;
    }
    
    // Write to file
    let output_path = openapi_dir.join("openapi.json");
    fs::write(&output_path, json)?;
    
    println!("OpenAPI specification generated successfully at: {}", output_path.display());
    
    Ok(())
}