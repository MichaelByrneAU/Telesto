use std::fs::File;
use std::io::Write;

use Result;

pub fn export(path: &Option<String>, contents: &str) -> Result<()> {
    match path {
        Some(path) => {
            let mut file = File::create(&path)?;
            file.write_all(contents.as_bytes())?;
        }
        None => println!("{}", contents),
    };
    Ok(())
}
