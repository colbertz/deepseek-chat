fn main() {
    use std::env;
    use std::fs;
    use std::path::Path;
    
    println!("cargo:info=Build script started");
    
    // Get directories
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let target_dir = env::var("CARGO_TARGET_DIR")
        .unwrap_or_else(|_| Path::new(&manifest_dir).join("target").to_str().unwrap().to_string());
    
    println!("cargo:info=MANIFEST_DIR: {}", manifest_dir);
    println!("cargo:info=PROFILE: {}", profile);
    println!("cargo:info=TARGET_DIR: {}", target_dir);
    
    // Binary output directory
    let bin_dir = Path::new(&target_dir).join(&profile);
    fs::create_dir_all(&bin_dir).expect("Failed to create bin directory");
    
    // Copy .env file if it exists
    let env_src = Path::new(&manifest_dir).join(".env");
    println!("cargo:info=Looking for .env at: {}", env_src.display());
    if env_src.exists() {
        let env_dest = bin_dir.join(".env");
        fs::copy(&env_src, &env_dest).expect("Failed to copy .env");
        println!("cargo:info=Copied .env to: {}", env_dest.display());
    } else {
        println!("cargo:info=.env not found at: {}", env_src.display());
    }

    // Copy database file
    let db_src = Path::new(&manifest_dir).join("sqlite/deepseek_chat.db");
    println!("cargo:info=Looking for DB at: {}", db_src.display());
    let db_dest = bin_dir.join("sqlite/deepseek_chat.db");

    // if sqlite not found, create the directory
    if !db_dest.parent().unwrap().exists() {
        fs::create_dir_all(db_dest.parent().unwrap()).expect("Failed to create DB directory");
        println!("cargo:info=Created DB directory: {}", db_dest.parent().unwrap().display());
    }
    fs::copy(&db_src, &db_dest).expect("Failed to copy database file");
    println!("cargo:info=Copied DB to: {}", db_dest.display());

    // Re-run if these files change
    println!("cargo:rerun-if-changed={}", env_src.display());
    println!("cargo:rerun-if-changed={}", db_src.display());
    println!("cargo:rerun-if-changed=build.rs");
}
