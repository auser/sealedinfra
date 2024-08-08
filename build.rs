extern crate vergen_gitcl;
use anyhow::Result;
use vergen_gitcl::*;

fn main() -> Result<()> {
    vergen()?;
    Ok(())
}

fn vergen() -> Result<()> {
    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::all_cargo()?;
    let git = GitclBuilder::all_git()?;
    let rustc = RustcBuilder::all_rustc()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&git)?
        .add_instructions(&rustc)?
        .emit()?;

    Ok(())
}
