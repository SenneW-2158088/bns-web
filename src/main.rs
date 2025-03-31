use std::error::Error;

mod low;

fn main() -> Result<(), Box<dyn Error>> {
<<<<<<< HEAD
    // low::idor::run()?;
    low::xss_reflected::run()?;
=======
    low::xss::run()?;
>>>>>>> 611dbf8 (ss_stored)

    Ok(())
}
