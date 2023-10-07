
use std::time::Duration;
use rand::random;
use mping;
use anyhow::Result;


fn main() -> Result<(), anyhow::Error>{
    let addr = "8.8.8.8".parse()?;
    let timeout = Duration::from_secs(1);
    mping::ping(addr, Some(timeout), Some(166), Some(3), Some(5), Some(&random()))?;

    Ok(())
}
