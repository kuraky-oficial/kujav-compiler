use std::fmt::Write as _;
use std::fs;

use crate::errors::KujavResult;
use crate::toml_config::KujavToml;

pub fn write_lockfile(cfg: &KujavToml) -> KujavResult<()> {
    let mut out = String::new();
    writeln!(&mut out, "version = 1").expect("write lockfile header");
    writeln!(&mut out, "").expect("write lockfile spacing");
    writeln!(&mut out, "[[package]]").expect("write lockfile package section");
    writeln!(&mut out, "name = \"{}\"", cfg.package.name).expect("write package name");
    writeln!(&mut out, "version = \"{}\"", cfg.package.version).expect("write package version");

    if !cfg.dependencies.is_empty() {
        writeln!(&mut out, "").expect("write dep spacing");
        for (name, version) in &cfg.dependencies {
            writeln!(&mut out, "[[dependency]]").expect("write dependency section");
            writeln!(&mut out, "name = \"{name}\"").expect("write dependency name");
            writeln!(&mut out, "version = \"{version}\"").expect("write dependency version");
            writeln!(&mut out, "").expect("write dependency spacing");
        }
    }

    fs::write("kujav.lock", out)?;
    Ok(())
}
