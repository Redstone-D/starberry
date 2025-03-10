use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf}; 
use std::process::{Command, exit};

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

fn main() { 
    // Skip the program name.
    let mut args: Vec<String> = env::args().skip(1).collect();

    // We expect the first argument to be "build"
    if args.is_empty() || args[0] != "build" {
        eprintln!("Usage: cargo myframework build [cargo build args]");
        exit(1);
    }
    // Remove "build" from our argument list so the rest will be passed to cargo build.
    args.remove(0);

    // Run "cargo build" with the remaining arguments.
    let status = Command::new("cargo")
        .arg("build")
        .args(&args)
        .status()
        .expect("Failed to run cargo build");
    if !status.success() {
        exit(status.code().unwrap_or(1));
    }

    // Determine whether to look for release or debug output.
    let is_release = args.iter().any(|arg| arg == "--release");
    // Use CARGO_TARGET_DIR if set, otherwise "target"
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
    let build_folder = if is_release { "release" } else { "debug" };
    let out_dir = Path::new(&target_dir).join(build_folder);

    // Our destination is a "templates" folder inside the output directory.
    let dest_templates = out_dir.join("templates");

    // Our source templates directory is assumed to be "./templates" in the crate root.
    let src_templates = Path::new("templates");
    if !src_templates.exists() {
        eprintln!("No templates directory found in crate root.");
        exit(1);
    }

    // Recursively copy the templates folder.
    if let Err(e) = copy_dir_all(src_templates, &dest_templates) {
        eprintln!("Error copying templates: {}", e);
        exit(1);
    }

    println!(
        "Successfully built the crate and copied templates to {}",
        dest_templates.display()
    );
}
