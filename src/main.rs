use std::error::Error;

mod low;
mod medium;
mod high;

fn main() -> Result<(), Box<dyn Error>> {
    medium::bruteforce::run();
    Ok(())
}
