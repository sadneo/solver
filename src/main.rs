use std::io;
use solver::evaluate;

pub fn main() -> anyhow::Result<()> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;

    let evaluation = evaluate(buffer.trim_end())?;
    println!("Evaluation: {}", evaluation);
    Ok(())
}
