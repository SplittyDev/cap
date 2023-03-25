use indicatif::ProgressBar;
use owo_colors::OwoColorize;
use std::{
    process::{Command, Stdio},
    time::Duration,
};

fn run_with_progress(command: &mut Command, message: String) -> anyhow::Result<()> {
    // Pipe stdout and stderr to parent
    command.stdout(Stdio::inherit());
    command.stderr(Stdio::piped());

    // Create progress bar
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_message(message);
    progress_bar.enable_steady_tick(Duration::from_millis(10));

    // Spawn command
    let mut child = command.spawn()?;

    // Wait for command to finish
    let status = child.wait()?;

    // Clear progress bar
    progress_bar.finish_and_clear();

    // Check for errors
    if !status.success() {
        anyhow::bail!("Failed to install package.");
    }

    Ok(())
}

pub fn install_package(
    package_name: impl AsRef<str>,
    version: semver::Version,
    locked: bool,
    forced: bool,
    nightly: bool,
) -> anyhow::Result<()> {
    let package_name = package_name.as_ref();

    let mut cargo = {
        let mut cargo = Command::new("cargo");
        if nightly {
            cargo.arg("+nightly");
        }
        cargo.arg("install");
        if forced {
            cargo.arg("--force");
        }
        if locked {
            cargo.arg("--locked");
        }
        cargo.arg(package_name);
        cargo
    };

    run_with_progress(
        &mut cargo,
        format!(
            "Installing package {} {}...",
            package_name.blue(),
            version.bright_black()
        ),
    )?;

    Ok(())
}

pub fn uninstall_package(package_name: impl AsRef<str>) -> anyhow::Result<()> {
    let package_name = package_name.as_ref();

    let output = Command::new("cargo")
        .arg("uninstall")
        .arg(package_name)
        .spawn()?
        .wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to uninstall package: {}", stderr);
    }

    Ok(())
}

pub fn update_package(
    package_name: impl AsRef<str>,
    local_version: semver::Version,
    target_version: semver::Version,
) -> anyhow::Result<()> {
    let package_name = package_name.as_ref();

    let mut cargo = Command::new("cargo");
    cargo
        .arg("install")
        .arg("--force")
        .arg("--version")
        .arg(target_version.to_string())
        .arg(package_name);

    run_with_progress(
        &mut cargo,
        format!(
            "{} package {} from {} to {}...",
            "Updating".green(),
            package_name.blue(),
            local_version.bright_black(),
            target_version.green()
        ),
    )?;

    Ok(())
}
