type E = Box<dyn std::error::Error + Send + Sync>;

fn main() -> Result<(), errors::Main> {
    one()?;
    Ok(())
}

fn one() -> Result<(), E> {
    println!("> one");
    two().map_err(|e| errors::wrap("two failed", e))?;
    println!("< one");
    Ok(())
}

fn two() -> Result<(), E> {
    println!("> two");
    Err(errors::new("kaboom").into())
}
