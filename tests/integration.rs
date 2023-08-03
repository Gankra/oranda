mod gallery;
use gallery::*;
use oranda::config::BoolOr;

#[test]
fn gal_axolotlsay() -> Result<()> {
    let test_name = _function_name!();
    AXOLOTLSAY.run_test(|ctx| {
        let mut config = ctx.load_oranda_json()?;
        ctx.save_oranda_json(config)?;

        let res = ctx.oranda_build(test_name)?;
        Ok(())
    })
}

#[test]
fn gal_cargo_dist() -> Result<()> {
    let test_name = _function_name!();
    CARGO_DIST.run_test(|ctx| {
        let mut config = ctx.load_oranda_json()?;
        ctx.save_oranda_json(config)?;

        let res = ctx.oranda_build(test_name)?;
        Ok(())
    })
}

#[test]
fn gal_akaikatana() -> Result<()> {
    let test_name = _function_name!();
    AKAIKATANA_REPACK.run_test(|ctx| {
        let mut config = ctx.load_oranda_json()?;
        // config.components.as_mut().unwrap().mdbook = Some(BoolOr::Bool(false));
        ctx.save_oranda_json(config)?;

        let res = ctx.oranda_build(test_name)?;
        Ok(())
    })
}
