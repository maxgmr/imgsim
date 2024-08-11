use vergen_gix::{BuildBuilder, CargoBuilder, Emitter, GixBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::all_cargo()?;
    let gitcl = GixBuilder::all_git()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&gitcl)?
        .emit()?;

    Ok(())
}
