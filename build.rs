use vergen::{BuildBuilder, CargoBuilder, Emitter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::all_cargo()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .emit()?;

    Ok(())
}
