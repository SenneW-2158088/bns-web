use std::error::Error;

mod low;
mod medium;

fn main() -> Result<(), Box<dyn Error>> {
    medium::bruteforce_2::run()?;
    Ok(())
}
