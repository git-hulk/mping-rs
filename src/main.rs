
use std::time::Duration;
use std::process;

use mping;
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "w", long = "timeout", default_value = "1")]
    timeout: u64,

    #[structopt(short = "t", long = "ttl", default_value = "64")]
    ttl: u32,

    #[structopt(short = "z", long = "tos")]
    tos: Option<u32>,

    #[structopt(short = "s", long = "size", default_value = "64")]
    size: usize,

    #[structopt(short = "r", long = "rate", default_value = "100")]
    rate: u64,

    #[structopt(short = "d", long = "delay", default_value = "3")]
    delay: u64,

    #[structopt(short = "c", long = "count")]
    count: Option<i32>,

    #[structopt(parse(from_os_str))]
    free: Vec<std::path::PathBuf>,
}

fn main() -> Result<(), anyhow::Error>{
    let opt = Opt::from_args();

    if opt.free.is_empty() {
        println!("Please input ip address");
        return Ok(());
    }

    let _ = opt.count;

    let addr = opt.free.last().unwrap().to_string_lossy().parse()?;
    let timeout = Duration::from_secs(opt.timeout);
    let pid = process::id() as u16;

    mping::ping(addr, timeout, opt.ttl, opt.tos, pid, opt.size,opt.rate,opt.delay)?;

    Ok(())
}
