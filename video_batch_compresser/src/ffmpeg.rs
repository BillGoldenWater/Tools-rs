use std::{ffi::OsStr, process::Command};

use anyhow::{Context as _, anyhow};

pub fn command(
    input: impl AsRef<OsStr>,
    args: &[impl AsRef<OsStr>],
    output: impl AsRef<OsStr>,
) -> Command {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-hide_banner")
        .arg("-y")
        .arg("-i")
        .arg(input)
        .args(args)
        .arg(output);
    cmd
}

pub fn run(
    input: impl AsRef<OsStr>,
    args: &[impl AsRef<OsStr>],
    output: impl AsRef<OsStr>,
) -> anyhow::Result<()> {
    let mut cmd = command(input, args, output);
    let out = cmd.output().context("failde to execute ffmpeg")?;

    if out.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "{}",
            String::from_utf8_lossy(&out.stderr).to_string()
        ))
        .context("ffmpeg exited with non zero status code")
    }
}
