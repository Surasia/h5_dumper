use crate::loader::H5Module;
use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use walkdir::WalkDir;

pub mod common;
mod loader;

/// Halo 5 module dumper.
/// Supports both Halo 5 Forge and Halo 5 campaign.
#[derive(Parser, Debug)]
#[command(version, about)]
struct H5ModuleLoader {
    /// Path to where modules are located (deploy folder).
    #[arg(short, long)]
    module_path: String,
    /// Path to save tags to.
    #[arg(short, long)]
    save_path: String,
}

fn read_module(file_name: &Path, save_path: &String) -> Result<()> {
    let file = File::open(file_name)?;
    let mut reader = BufReader::new(file);
    let mut module = H5Module::default();

    module.read(&mut reader)?;
    for file in module.files {
        let file_p = Path::new("..")
            .join(save_path)
            .join(file.name.replace(":", "_").replace("*", "_"));

        std::fs::create_dir_all(file_p.parent().unwrap())?;
        let mut handle = File::create(file_p)?;
        handle.write_all(&file.data)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let arguments = H5ModuleLoader::parse();
    for file in WalkDir::new(arguments.module_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if file.path().to_str().unwrap().ends_with("module") {
            println!("Dumping module: {}", file.path().to_str().unwrap());
            read_module(file.path(), &arguments.save_path)?;
        }
    }
    Ok(())
}
