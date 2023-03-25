use std::process::Command;

pub fn install_package(
    package_name: impl AsRef<str>,
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

    let output = cargo.spawn()?.wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to install package: {}", stderr);
    }

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
    target_version: semver::Version,
) -> anyhow::Result<()> {
    let package_name = package_name.as_ref();

    let output = Command::new("cargo")
        .arg("install")
        .arg("--force")
        .arg("--version")
        .arg(target_version.to_string())
        .arg(package_name)
        .spawn()?
        .wait_with_output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to update package: {}", stderr);
    }

    Ok(())
}
