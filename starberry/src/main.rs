use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

static VERSION: &str = env!("CARGO_PKG_VERSION"); 

/// Recursively copy all files and subdirectories from `src` to `dst`.
fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            fs::copy(&entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

/// Launches a cargo command with the given command name and arguments.
/// Returns the exit status.
fn run_cargo(cmd: &str, args: &[String]) -> i32 {
    let status = Command::new("cargo")
        .arg(cmd)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to run cargo {}: {}", cmd, e);
            exit(1);
        });
    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
    status.code().unwrap_or(0)
} 

/// Copies required assets (templates and programfiles) to the target folder after a build.
fn copy_assets(is_release: bool) {
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let build_folder = if is_release { "release" } else { "debug" };
    let out_dir = Path::new(&target_dir).join(build_folder);
    
    // Define directories to copy (templates is required, programfiles is optional)
    let dirs = [
        ("templates", false),   // (directory name, not required)
        ("programfiles", true), // (directory name, required) 
    ];

    for (dir_name, required) in dirs {
        let src_dir = Path::new(dir_name);
        let dest_dir = out_dir.join(dir_name);

        // Handle missing directories based on requirement
        if !src_dir.exists() {
            if required {
                eprintln!("Error: required directory '{}' not found in crate root", dir_name);
                exit(1);
            }
            continue; // Skip optional missing directories
        }

        // Perform the copy
        if let Err(e) = copy_dir_all(src_dir, &dest_dir) {
            eprintln!("Error copying {}: {}", dir_name, e);
            exit(1);
        }
        
        println!("Successfully copied {} to {}", dir_name, dest_dir.display());
    }
} 

/// Creates a new project with the given app name.
/// This function calls `cargo new <app_name>`, then creates a default main.rs,
/// updates Cargo.toml with extra dependencies, and creates a new templates directory
/// at the same level as the src folder.
fn create_new_project(app_name: &str) {
    // Run `cargo new <app_name>`
    let status = Command::new("cargo")
        .arg("new")
        .arg(app_name)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("Failed to run cargo new: {}", e);
            exit(1);
        });
    if !status.success() {
        exit(status.code().unwrap_or(1));
    }

    // Define the new main.rs content using Starberry sample code.
    let main_rs_content = r#"use starberry::prelude::*;

#[tokio::main]
async fn main() {
    APP.clone().run().await;
}

pub static APP: SApp = once_cell::sync::Lazy::new(|| {
    App::new().build()
});

#[url(APP.lit_url("/"))] 
async fn home_route() -> HttpResponse {
    text_response("Hello, world!")
}
"#;

    // Write the new main.rs to the src directory of the new project.
    let src_path = Path::new(app_name).join("src").join("main.rs");
    fs::write(&src_path, main_rs_content).unwrap_or_else(|e| {
        eprintln!("Failed to write to {}: {}", src_path.display(), e);
        exit(1);
    });
    println!("Created new main.rs at {}", src_path.display());

    // Update Cargo.toml with the extra dependencies.
    let cargo_toml_path = Path::new(app_name).join("Cargo.toml");
    let deps = format!(
        r#"starberry = "{VERSION}"
ctor = "0.4.0"
once_cell = "1.17"
tokio = {{ version = "1", features = ["full"] }}
"#, 
    ); 
    let mut file = OpenOptions::new()
        .append(true)
        .open(&cargo_toml_path)
        .unwrap_or_else(|e| {
            eprintln!("Failed to open {}: {}", cargo_toml_path.display(), e);
            exit(1);
        });
    writeln!(file, "{}", deps).unwrap_or_else(|e| {
        eprintln!("Failed to write to {}: {}", cargo_toml_path.display(), e);
        exit(1);
    });
    println!("Updated Cargo.toml with extra dependencies.");

    // Create a new templates directory at the same level as src.
    let templates_path = Path::new(app_name).join("templates");
    if let Err(e) = fs::create_dir_all(&templates_path) {
        eprintln!("Failed to create templates directory: {}", e);
        exit(1);
    } 

    // Create a new program files directory at the same level as src.
    let templates_path = Path::new(app_name).join("templates");
    if let Err(e) = fs::create_dir_all(&templates_path) {
        eprintln!("Failed to create templates directory: {}", e);
        exit(1);
    }
    println!("Created templates directory at {}", templates_path.display());
}

/// Main entry point for the CLI launcher.
/// 
/// # Commands
/// 
/// - `build`: Runs `cargo build` with any extra arguments, then copies templates.
/// - `run`: Runs `cargo run` with any extra arguments.
/// - `release`: Runs `cargo build --release` with any extra arguments, then copies templates.
/// - `new <app_name>`: Creates a new project with the given name, writes a default `main.rs`
///   with Starberry code, updates `Cargo.toml` with dependencies, and creates a new templates directory.
/// 
/// # Example Usage
/// 
/// Build a project:
/// 
/// ```bash
/// starberry build --verbose
/// ```
/// 
/// Run a project:
/// 
/// ```bash
/// starberry run
/// ```
/// 
/// Build a release version:
/// 
/// ```bash
/// starberry release --release
/// ```
/// 
/// Create a new project called `my_app`:
/// 
/// ```bash
/// starberry new my_app
/// ```
fn main() {
    // Skip the program name.
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: starberry <command> [arguments]");
        eprintln!(r#"Usage: starberry <build|run|release|new|version> [arguments]
- `new <app_name>`: Creates a new project with the given name, a hello world program is provided by default. Dependencies are added to the Cargo.toml file. A templates directory is created at the same level as src. 
- `build [arguments]`: Build the Starberry project (Do not use cargo build since it does not copies template). Any other extra arguments are passed to `cargo build`. 
- `run`: Runs the starberry project. 
- `release`: Build the Starberry project in release mode (Do not use cargo build --release since it does not copies template). Any other extra arguments are passed to `cargo build`.  
- `version`: Prints the version of Starberry. 
"#);
        exit(1);
    }
    
    // Extract the command.
    let command = args.remove(0);
    
    match command.as_str() {
        "build" => {
            // Run cargo build with remaining arguments.
            let exit_code = run_cargo("build", &args);
            // Determine whether the build is release.
            let is_release = args.iter().any(|arg| arg == "--release");
            // Copy templates after a successful build.
            copy_assets(is_release);
            exit(exit_code);
        },
        "run" => {
            // Run cargo run with remaining arguments.
            let exit_code = run_cargo("run", &args);
            exit(exit_code); 
        },
        "release" => {
            // Ensure that --release flag is passed.
            if !args.iter().any(|arg| arg == "--release") {
                args.push("--release".to_string());
            }
            let exit_code = run_cargo("build", &args);
            copy_assets(true);
            exit(exit_code);
        },
        "new" => {
            if args.is_empty() {
                eprintln!("Usage: starberry new <app_name>");
                exit(1);
            }
            let app_name = &args[0];
            create_new_project(app_name);
        }, 
        "version" => {
            println!("Starberry version: {}", VERSION); 
            exit(0); 
        }, 
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!(r#"Usage: starberry <build|run|release|new> [arguments]
- `new <app_name>`: Creates a new project with the given name, a hello world program is provided by default. Dependencies are added to the Cargo.toml file. A templates directory is created at the same level as src. 
- `build [arguments]`: Build the Starberry project (Do not use cargo build since it does not copies template). Any other extra arguments are passed to `cargo build`. 
- `run`: Runs the starberry project. 
- `release`: Build the Starberry project in release mode (Do not use cargo build --release since it does not copies template). Any other extra arguments are passed to `cargo build`.  
- `version`: Prints the version of Starberry. 
"#);
            exit(1); 
        }
    }
}
