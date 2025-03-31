use std::error::Error;

mod low;

fn main() -> Result<(), Box<dyn Error>> {
    low::idor::run()?;

    Ok(())
}
