use anyhow::Result;
use std::path::Path;

pub fn assess(target: &str, policy: Option<&Path>) -> Result<()> {
    match policy {
        Some(path) => println!("assessing {target} against policy {}", path.display()),
        None => println!("assessing {target} against default policy"),
    }
    Ok(())
}
