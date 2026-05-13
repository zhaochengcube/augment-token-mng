use crate::antigravity::modules::process;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct AntigravityVersion {
    pub short_version: String,
}

pub fn get_antigravity_version() -> Result<AntigravityVersion, String> {
    get_antigravity_version_for_path(None)
}

pub fn get_antigravity_version_for_path(
    custom_executable_path: Option<&str>,
) -> Result<AntigravityVersion, String> {
    let exe_path = resolve_executable_path(custom_executable_path)?;

    #[cfg(target_os = "windows")]
    {
        get_version_windows(&exe_path)
    }

    #[cfg(target_os = "macos")]
    {
        get_version_macos(&exe_path)
    }

    #[cfg(target_os = "linux")]
    {
        get_version_linux(&exe_path)
    }
}

pub fn is_new_version(version: &AntigravityVersion) -> bool {
    compare_version(&version.short_version, "1.16.5") >= std::cmp::Ordering::Equal
}

fn resolve_executable_path(custom_executable_path: Option<&str>) -> Result<PathBuf, String> {
    if let Some(path) = custom_executable_path {
        let custom = PathBuf::from(path);
        if custom.exists() {
            return Ok(custom);
        }
    }

    process::get_antigravity_executable_path()
        .map_err(|e| format!("Unable to locate Antigravity executable: {}", e))
}

#[cfg(target_os = "windows")]
fn get_version_windows(exe_path: &PathBuf) -> Result<AntigravityVersion, String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("(Get-Item '{}').VersionInfo.FileVersion", exe_path.display()),
        ])
        .output()
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

    if !output.status.success() {
        return Err("Failed to read version from executable".to_string());
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version = extract_semver(&raw).ok_or_else(|| {
        format!("Version information not found in executable output: {}", raw)
    })?;

    Ok(AntigravityVersion {
        short_version: version,
    })
}

#[cfg(target_os = "macos")]
fn get_version_macos(_exe_path: &PathBuf) -> Result<AntigravityVersion, String> {
    Err("Antigravity version detection is not implemented on macOS".to_string())
}

#[cfg(target_os = "linux")]
fn get_version_linux(exe_path: &PathBuf) -> Result<AntigravityVersion, String> {
    let output = Command::new(exe_path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute Antigravity --version: {}", e))?;

    if !output.status.success() {
        return Err("Failed to read version from executable".to_string());
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version = extract_semver(&raw).ok_or_else(|| {
        format!("Version information not found in executable output: {}", raw)
    })?;

    Ok(AntigravityVersion {
        short_version: version,
    })
}

fn extract_semver(raw: &str) -> Option<String> {
    for token in raw.split(|c: char| c.is_whitespace() || c == ',' || c == ';') {
        let trimmed = token.trim_matches(|c: char| {
            c == '"' || c == '\'' || c == '(' || c == ')' || c == '[' || c == ']'
        });
        if trimmed.is_empty() {
            continue;
        }

        let numeric_parts: Vec<&str> = trimmed
            .split('.')
            .filter(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
            .collect();

        if numeric_parts.len() >= 3 {
            return Some(numeric_parts[..3].join("."));
        }
    }

    None
}

fn compare_version(v1: &str, v2: &str) -> std::cmp::Ordering {
    let parts1: Vec<u32> = v1.split('.').filter_map(|s| s.parse().ok()).collect();
    let parts2: Vec<u32> = v2.split('.').filter_map(|s| s.parse().ok()).collect();

    for idx in 0..parts1.len().max(parts2.len()) {
        let p1 = *parts1.get(idx).unwrap_or(&0);
        let p2 = *parts2.get(idx).unwrap_or(&0);
        match p1.cmp(&p2) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    std::cmp::Ordering::Equal
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_semver_handles_four_segment_file_version() {
        assert_eq!(extract_semver("1.16.5.0"), Some("1.16.5".to_string()));
        assert_eq!(extract_semver("version 1.107.0"), Some("1.107.0".to_string()));
    }

    #[test]
    fn compare_version_works_for_threshold() {
        let old = AntigravityVersion {
            short_version: "1.16.4".to_string(),
        };
        let new = AntigravityVersion {
            short_version: "1.16.5".to_string(),
        };

        assert!(!is_new_version(&old));
        assert!(is_new_version(&new));
    }
}
