use std::error::Error;

mod low;
mod medium;

fn main() -> Result<(), Box<dyn Error>> {
<<<<<<< HEAD
    medium::bruteforce_2::run()?;
=======
    medium::bruteforce::run();
>>>>>>> e844e98 (merging)
    Ok(())
}
