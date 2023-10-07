
use std::time::Duration;
use std::process;

use mping;
use anyhow::Result;


fn main() -> Result<(), anyhow::Error>{
    let addr = "8.8.8.8".parse()?;
    let timeout = Duration::from_secs(1);

    let pid = process::id() as u16;

    mping::ping(addr, Some(timeout), Some(166), Some(pid), b"hello world")?;

    Ok(())
}
