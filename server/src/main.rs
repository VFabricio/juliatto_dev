mod config;

use anyhow::Result;

use self::config::AppConfig;

fn main() -> Result<()> {
    let config = AppConfig::load()?;
    println!("{config:?}");
    Ok(())
}
