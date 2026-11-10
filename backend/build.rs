// extern crate vergen;

// use vergen::{gen, ConstantsFlags};

use std::{env, fs, path::Path};

fn main() {
    // Setup the flags, toggling off the 'SEMVER_FROM_CARGO_PKG' flag
    // let mut flags = ConstantsFlags::all();
    // flags.toggle(ConstantsFlags::SEMVER_FROM_CARGO_PKG);

    // // Generate the 'cargo:' key output
    // gen(ConstantsFlags::all()).expect("Unable to generate the cargo keys!");
    println!("cargo::rustc-cfg=dev");

    embed_migrations();
}

/// Compile every `migrations/<version>_<description>.sql` into the binary, so
/// the backend can apply them at startup without the files being deployed
/// next to it (see `src/migrate.rs`). Same naming rule as
/// `migrations/run_migrations.py`: a numeric version, an underscore, a
/// description; anything else in the directory is ignored.
fn embed_migrations() {
    let dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("migrations");
    // Re-run when a migration is added, removed or edited.
    println!("cargo::rerun-if-changed={}", dir.display());

    let mut found: Vec<(u64, String, String)> = Vec::new();
    for entry in fs::read_dir(&dir).expect("read backend/migrations") {
        let path = entry.expect("read backend/migrations entry").path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(stem) = name.strip_suffix(".sql") else {
            continue;
        };
        let Some((version, description)) = stem.split_once('_') else {
            continue;
        };
        if description.is_empty() || !version.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let Ok(version) = version.parse::<u64>() else {
            continue;
        };
        println!("cargo::rerun-if-changed={}", path.display());
        found.push((version, description.to_string(), path.display().to_string()));
    }
    found.sort();

    let mut code = String::from("&[\n");
    for (version, description, path) in &found {
        code.push_str(&format!(
            "    Migration {{ version: {version}, description: {description:?}, sql: include_bytes!({path:?}) }},\n"
        ));
    }
    code.push_str("]\n");
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("migrations.rs");
    fs::write(out, code).expect("write migrations.rs");
}
