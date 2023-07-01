use std::{
    env::{split_paths, var_os},
    ffi::OsString,
    io::BufRead,
    ops::Range,
    path::PathBuf,
    process::Command,
};

use anyhow::{bail, Result};

pub struct Context {
    pub(crate) builtins: Vec<OsString>,
    pub(crate) patches: Vec<(Range<usize>, PathBuf)>,
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) src: Vec<u8>,
    pub(crate) store_dir: PathBuf,
}

impl Context {
    pub fn load(
        bash: OsString,
        path: Option<OsString>,
        src: Vec<u8>,
        store_dir: PathBuf,
    ) -> Result<Self> {
        Ok(Context {
            builtins: load_builtins(bash)?,
            patches: Vec::new(),
            paths: load_paths(path),
            src,
            store_dir,
        })
    }
}

fn load_builtins(bash: OsString) -> Result<Vec<OsString>> {
    let output = Command::new(&bash).arg("-c").arg("enable").output()?;

    if !output.status.success() {
        bail!(
            "command `{} -c enable` failed: {}\n\nstdout: {}\nstderr: {}",
            bash.to_string_lossy(),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    Ok(output
        .stdout
        .lines()
        .filter_map(|line| line.ok()?.strip_prefix("enable ").map(Into::into))
        .collect())
}

fn load_paths(path: Option<OsString>) -> Vec<PathBuf> {
    path.or_else(|| var_os("PATH"))
        .map_or_else(Vec::new, |path| split_paths(&path).collect())
}
