
use std::time::Duration;
use rand::random;


fn main(){
    let addr = "127.0.0.1".parse().unwrap();
    let timeout = Duration::from_secs(1);
    mping::ping(addr, Some(timeout), Some(166), Some(3), Some(5), Some(&random())).unwrap();
}
