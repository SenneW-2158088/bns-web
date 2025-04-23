use std::error::Error;

mod low;
mod medium;
mod high;

fn main() -> Result<(), Box<dyn Error>> {
    high::xss_stored_server::run();
    Ok(())
}
