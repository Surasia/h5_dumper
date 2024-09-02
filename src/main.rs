use crate::loader::H5Module;
use anyhow::Result;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

pub mod common;
mod loader;

fn main() -> Result<()> {
    let file = File::open("")?;
    let mut reader = BufReader::new(file);
    let mut module = H5Module::default();

    module.read(&mut reader)?;
    for file in module.files {
        println!("{}", file.name);
        let path = format! {"{}{}", "", file.name.replace(":", "_").replace("*", "_")};
        let file_p = Path::new(&path);
        std::fs::create_dir_all(file_p.parent().unwrap())?;
        let mut handle = File::create(file_p)?;
        handle.write_all(&file.data)?;
    }

    Ok(())
}
