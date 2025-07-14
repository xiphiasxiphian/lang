mod frontend
{
    pub mod parsers;
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let a = 5;
    println!("{:p}", &a);

    let a = 5;
    println!("{:p}", &a);

    Ok(())
}
