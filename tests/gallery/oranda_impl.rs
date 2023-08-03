use std::collections::BTreeMap;
use std::sync::Mutex;

use axoasset::LocalAsset;
use camino::{Utf8Path, Utf8PathBuf};
use miette::{miette, IntoDiagnostic};
use oranda::config::{OrandaLayer, WorkspaceLayer, WorkspaceMember};

use super::command::CommandInfo;
use super::errors::Result;
use super::repo::{Repo, TestContext, TestContextLock, ToolsImpl};

/// Set this at runtime to override STATIC_CARGO_DIST_BIN
const ENV_RUNTIME_ORANDA_BIN: &str = "OVERRIDE_CARGO_BIN_EXE_oranda";
const STATIC_ORANDA_BIN: &str = env!("CARGO_BIN_EXE_oranda");
const ROOT_DIR: &str = env!("CARGO_MANIFEST_DIR");
static TOOLS: Mutex<Option<Tools>> = Mutex::new(None);

/// axolotlsay 0.1.0 is a nice simple project with shell+powershell+npm installers in its release
pub static AXOLOTLSAY: TestContextLock<Tools> = TestContextLock::new(
    &TOOLS,
    &Repo {
        repo_owner: "axodotdev",
        repo_name: "axolotlsay",
        commit_sha: "8fa45257dbcb7b45e23e62660b01fd2d71676a9d",
        app_name: "axolotlsay",
        subdir: None,
        bins: &["axolotlsay"],
    },
);
/// akaikatana-repack 0.2.0 has multiple bins!
pub static AKAIKATANA_REPACK: TestContextLock<Tools> = TestContextLock::new(
    &TOOLS,
    &Repo {
        repo_owner: "mistydemeo",
        repo_name: "akaikatana-repack",
        commit_sha: "9516f77ab81b7833e0d66de766ecf802e056f91f",
        app_name: "akaikatana-repack",
        subdir: None,
        bins: &["akextract", "akmetadata", "akrepack"],
    },
);
/// it's cargo-dist!
pub static CARGO_DIST: TestContextLock<Tools> = TestContextLock::new(
    &TOOLS,
    &Repo {
        repo_owner: "axodotdev",
        repo_name: "cargo-dist",
        commit_sha: "d4df220226f306d082f656c29e247a959b564e1b",
        app_name: "cargo-dist",
        subdir: Some("cargo-dist"),
        bins: &["cargo-dist"],
    },
);
/// it's oranda!
pub static ORANDA: TestContextLock<Tools> = TestContextLock::new(
    &TOOLS,
    &Repo {
        repo_owner: "axodotdev",
        repo_name: "oranda",
        commit_sha: "489db7b6a83a463ee256c9297aa97661fb844bea",
        app_name: "oranda",
        subdir: None,
        bins: &["oranda"],
    },
);

pub struct Tools {
    pub git: CommandInfo,
    pub oranda: CommandInfo,
}

impl Tools {
    fn new() -> Self {
        eprintln!("getting tools...");
        let git = CommandInfo::new("git", None).expect("git isn't installed");

        // If OVERRIDE_* is set, prefer that over the version that cargo built for us,
        // this lets us test our shippable builds.
        let oranda_path =
            std::env::var(ENV_RUNTIME_ORANDA_BIN).unwrap_or_else(|_| STATIC_ORANDA_BIN.to_owned());
        let oranda = CommandInfo::new("oranda", Some(&oranda_path)).expect("oranda isn't built!?");

        const TARGET_TEMP_DIR: &str = env!("CARGO_TARGET_TMPDIR");
        let public = Utf8Path::new(TARGET_TEMP_DIR).join("public");
        let workspace_json = Utf8Path::new(TARGET_TEMP_DIR).join("oranda-workspace.json");
        if public.exists() {
            std::fs::remove_dir_all(public).unwrap();
        }
        let json = OrandaLayer {
            project: None,
            build: None,
            marketing: None,
            styles: None,
            components: None,
            workspace: Some(WorkspaceLayer {
                name: Some("oranda gallery".to_owned()),
                generate_index: Some(true),
                members: Some(vec![]),
                auto: Some(false),
            }),
            _schema: None,
        };
        let json_src = serde_json::to_string_pretty(&json).unwrap();
        axoasset::LocalAsset::write_new(&json_src, &workspace_json).unwrap();

        Self { git, oranda }
    }
}

impl ToolsImpl for Tools {
    fn git(&self) -> &CommandInfo {
        &self.git
    }
}
impl Default for Tools {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OrandaResult {
    test_name: String,
}

impl<'a> TestContext<'a, Tools> {
    /// Run 'cargo dist build -aglobal' with the toml patched
    /// and return paths to various files that were generated
    pub fn oranda_build(&self, test_name: &str) -> Result<OrandaResult> {
        eprintln!("\n=============== running test: {test_name} =================");

        // build installers
        eprintln!("running oranda build...");
        self.tools.oranda.output_checked(|cmd| cmd.arg("build"))?;

        self.load_oranda_results(test_name)
    }

    fn load_oranda_results(&self, test_name: &str) -> Result<OrandaResult> {
        // read/analyze installers
        eprintln!("loading results...");
        let app_name = &self.repo.app_name;

        /*
               let tmp = self.repo_dir.parent().unwrap();
               let src = Utf8Path::new("public");
               let new_public = tmp.join("public");
               let dest = new_public.join(test_name);
               std::fs::create_dir_all(&new_public).into_diagnostic()?;
               std::fs::rename(&src, &dest).into_diagnostic()?;
        */
        let tmp = self.repo_dir.parent().unwrap();
        let workspace_json = tmp.join("oranda-workspace.json");
        let json_src = axoasset::SourceFile::load_local(&workspace_json)?;
        let mut json: OrandaLayer = json_src.deserialize_json()?;
        let path = if let Some(subdir) = &self.repo.subdir {
            Utf8Path::new(&self.repo_id).join(subdir)
        } else {
            Utf8Path::new(&self.repo_id).to_owned()
        };
        json.workspace
            .as_mut()
            .unwrap()
            .members
            .as_mut()
            .unwrap()
            .push(WorkspaceMember {
                slug: test_name.to_owned(),
                path: path.into_std_path_buf(),
            });
        let json_src = serde_json::to_string_pretty(&json).into_diagnostic()?;
        axoasset::LocalAsset::write_new(&json_src, &workspace_json)?;

        Ok(OrandaResult {
            test_name: test_name.to_owned(),
        })
    }

    pub fn load_oranda_json(&self) -> Result<oranda::config::OrandaLayer> {
        eprintln!("loading oranda.json...");
        let json_src = axoasset::SourceFile::load_local("oranda.json")?;
        let json = json_src.deserialize_json()?;
        Ok(json)
    }
    pub fn save_oranda_json(&self, json: oranda::config::OrandaLayer) -> Result<()> {
        eprintln!("storing oranda.json...");
        let json_src = serde_json::to_string_pretty(&json).into_diagnostic()?;
        axoasset::LocalAsset::write_new(&json_src, "oranda.json")?;
        Ok(())
    }
}

impl OrandaResult {
    pub fn check_all(&self, ctx: &TestContext<Tools>, expected_bin_dir: &str) -> Result<()> {
        // Now that all other checks have passed, it's safe to check snapshots
        self.snapshot()?;

        Ok(())
    }

    // Run cargo-insta on everything we care to snapshot
    pub fn snapshot(&self) -> Result<()> {
        // We make a single uber-snapshot to avoid the annoyances of having multiple snapshots in one test
        let mut snapshots = String::new();

        let test_name = &self.test_name;
        snapshot_settings().bind(|| {
            insta::assert_snapshot!(format!("{test_name}-installers"), &snapshots);
        });
        Ok(())
    }

    fn append_snapshot_file(
        out: &mut String,
        name: &str,
        src_path: Option<&Utf8Path>,
    ) -> Result<()> {
        // Skip snapshotting this file if absent
        let Some(src_path) = src_path else {
            return Ok(());
        };

        let src = axoasset::LocalAsset::load_string(src_path)?;
        Self::append_snapshot_string(out, name, &src)
    }

    fn append_snapshot_string(out: &mut String, name: &str, val: &str) -> Result<()> {
        use std::fmt::Write;

        writeln!(out, "================ {name} ================").unwrap();
        writeln!(out, "{val}").unwrap();
        Ok(())
    }
}

pub fn snapshot_settings() -> insta::Settings {
    let mut settings = insta::Settings::clone_current();
    let snapshot_dir = Utf8Path::new(ROOT_DIR).join("tests").join("snapshots");
    settings.set_snapshot_path(snapshot_dir);
    settings.set_prepend_module_to_snapshot(false);
    settings
}

pub fn snapshot_settings_with_version_filter() -> insta::Settings {
    let mut settings = snapshot_settings();
    settings.add_filter(
        r"\d+\.\d+\.\d+(\-prerelease\d*)?(\.\d+)?",
        "1.0.0-FAKEVERSION",
    );
    settings
}

#[allow(dead_code)]
pub fn snapshot_settings_with_dist_manifest_filter() -> insta::Settings {
    let mut settings = snapshot_settings_with_version_filter();
    settings.add_filter(
        r#""announcement_tag": .*"#,
        r#""announcement_tag": "CENSORED","#,
    );
    settings.add_filter(
        r#""announcement_title": .*"#,
        r#""announcement_title": "CENSORED""#,
    );
    settings.add_filter(
        r#""announcement_changelog": .*"#,
        r#""announcement_changelog": "CENSORED""#,
    );
    settings.add_filter(
        r#""announcement_github_body": .*"#,
        r#""announcement_github_body": "CENSORED""#,
    );
    settings.add_filter(
        r#""announcement_is_prerelease": .*"#,
        r#""announcement_is_prerelease": "CENSORED""#,
    );
    settings.add_filter(
        r#""cargo_version_line": .*"#,
        r#""cargo_version_line": "CENSORED""#,
    );
    settings
}
