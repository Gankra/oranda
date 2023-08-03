mod gallery;
use std::{env::set_current_dir, collections::BTreeSet};

use camino::Utf8Path;
use gallery::*;
use oranda::config::{BoolOr, OrandaLayer};

#[test]
fn gal_axolotlsay() -> Result<()> {
    let test_name = _function_name!();
    AXOLOTLSAY.run_test(|ctx| {
        ctx.oranda_build(test_name)?;
        Ok(())
    })
}

#[test]
fn gal_cargo_dist() -> Result<()> {
    let test_name = _function_name!();
    CARGO_DIST.run_test(|ctx| {
        let mut config = ctx.load_oranda_json()?;
        // config.components.as_mut().unwrap().mdbook = Some(BoolOr::Bool(false));
        ctx.save_oranda_json(config)?;

        ctx.oranda_build(test_name)?;
        Ok(())
    })
}

#[test]
fn gal_akaikatana() -> Result<()> {
    let test_name = _function_name!();
    AKAIKATANA_REPACK.run_test(|ctx| {
        ctx.oranda_build(test_name)?;
        Ok(())
    })
}

#[test]
fn gal_oranda() -> Result<()> {
    let test_name = _function_name!();
    ORANDA.run_test(|ctx| {
        ctx.oranda_build(test_name)?;
        Ok(())
    })
}


#[test]
fn gal_workspace() -> Result<()> {
    let test_name = _function_name!();
    loop {
        // Bail out and sleep for a while if not all the other tests are written
        let mut should_sleep = true;
        AXOLOTLSAY.run_test(|ctx| {
            // Go to the root
            let tmp = ctx.repo_dir.parent().unwrap();
            set_current_dir(tmp).unwrap();

            // Load the oranda-workspace.json and check if all tests are done
            let workspace_json = Utf8Path::new("oranda-workspace.json");
            let json_src = axoasset::SourceFile::load_local(&workspace_json)?;
            let json: OrandaLayer = json_src.deserialize_json()?;
            let members = json.workspace.as_ref().unwrap().members.as_ref().unwrap();
            let members_set = members.iter().map(|m| m.slug.clone()).collect::<BTreeSet<String>>();
            let required_set = vec![
                "gal_cargo_dist".to_owned(),
                "gal_axolotlsay".to_owned(),
                "gal_akaikatana".to_owned(),
                "gal_oranda".to_owned(),
            ].into_iter().collect::<BTreeSet<String>>();
    
            if !required_set.is_subset(&members_set) {
                // Sleep
                return Ok(())
            }
            should_sleep = false;

            ctx.oranda_build(test_name)?;
            Ok(())
        })?;

        if should_sleep {
            std::thread::sleep(std::time::Duration::from_secs(1));
        } else {
            return Ok(())
        }
    }
    

}