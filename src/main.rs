type Result<T> = std::result::Result<T, std::boxed::Box<dyn std::error::Error>>;
type UnitResult = Result<()>;

fn run() -> UnitResult {
    Ok(())
}

fn main() {
    run().expect("runtime error");
}
