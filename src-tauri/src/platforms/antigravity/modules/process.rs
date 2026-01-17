use std::process::Command;
use std::path::PathBuf;
use sysinfo::{System, ProcessRefreshKind, ProcessesToUpdate};
use super::config;

fn is_helper_process(name: &str, args: &str) -> bool {
    let name = name.to_lowercase();
    let args = args.to_lowercase();
    args.contains("--type=")
        || name.contains("helper")
        || name.contains("plugin")
        || name.contains("renderer")
        || name.contains("gpu")
        || name.contains("crashpad")
        || name.contains("utility")
        || name.contains("audio")
        || name.contains("sandbox")
        || name.contains("language_server")
}

fn is_antigravity_process(process: &sysinfo::Process) -> bool {
    let name = process.name().to_string_lossy().to_lowercase();
    if name.contains("antigravity") {
        return true;
    }

    let exe_path = process
        .exe()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_lowercase();

    #[cfg(target_os = "macos")]
    {
        return exe_path.contains("antigravity.app");
    }

    #[cfg(target_os = "windows")]
    {
        return exe_path.ends_with("antigravity.exe");
    }

    #[cfg(target_os = "linux")]
    {
        return exe_path.contains("/antigravity");
    }
}

fn get_antigravity_pids() -> Vec<u32> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::new());

    let mut pids = Vec::new();
    for (pid, process) in sys.processes() {
        if is_antigravity_process(process) {
            pids.push(pid.as_u32());
        }
    }
    pids
}

/// 检查 Antigravity 是否正在运行
pub fn is_antigravity_running() -> bool {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::new());

    sys.processes().values().any(is_antigravity_process)
}

/// 温和关闭 Antigravity（优先主进程）
pub fn close_antigravity(timeout_secs: u64) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let pids = get_antigravity_pids();
        if pids.is_empty() {
            return Ok(());
        }

        // macOS: try a clean quit to avoid "window terminated unexpectedly"
        let _ = Command::new("osascript")
            .args(["-e", "tell application \"Antigravity\" to quit"])
            .output();
        let quit_wait = std::cmp::min(timeout_secs, 3);
        let start_quit = std::time::Instant::now();
        while start_quit.elapsed() < std::time::Duration::from_secs(quit_wait) {
            if !is_antigravity_running() {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        let mut sys = System::new();
        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::new());

        let mut main_pid: Option<u32> = None;
        for pid_u32 in &pids {
            let pid = sysinfo::Pid::from_u32(*pid_u32);
            if let Some(process) = sys.process(pid) {
                let name = process.name().to_string_lossy();
                let args = process
                    .cmd()
                    .iter()
                    .map(|arg| arg.to_string_lossy().into_owned())
                    .collect::<Vec<String>>()
                    .join(" ");

                if !is_helper_process(&name, &args) {
                    main_pid = Some(*pid_u32);
                    break;
                }
            }
        }

        if let Some(pid) = main_pid {
            let _ = Command::new("kill").args(["-15", &pid.to_string()]).output();
        } else {
            for pid in &pids {
                let _ = Command::new("kill").args(["-15", &pid.to_string()]).output();
            }
        }

        let graceful_timeout = (timeout_secs * 7) / 10;
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(graceful_timeout) {
            if !is_antigravity_running() {
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(300));
        }

        if is_antigravity_running() {
            let remaining = get_antigravity_pids();
            for pid in remaining {
                let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        kill_antigravity_processes().ok();
        Ok(())
    }
}

/// 杀死所有 Antigravity 进程
pub fn kill_antigravity_processes() -> Result<(), String> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::new());

    let mut killed_count = 0;

    for (pid, process) in sys.processes() {
        if is_antigravity_process(process) {
            if process.kill() {
                killed_count += 1;
                println!("Killed Antigravity process: {} (PID: {})", process.name().to_string_lossy(), pid);
            }
        }
    }

    if killed_count > 0 {
        // 等待进程完全退出
        std::thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    } else {
        Err("No Antigravity processes found".to_string())
    }
}

/// 获取 Antigravity 可执行文件路径（支持自定义路径）
pub fn get_antigravity_executable_path_with_config(
    app_handle: Option<&tauri::AppHandle>
) -> Result<PathBuf, String> {
    // 如果提供了 app_handle，尝试加载配置
    if let Some(handle) = app_handle {
        if let Ok(config) = config::load_config(handle) {
            // 如果启用了自定义路径
            if config.use_custom_path {
                if let Some(custom_path) = config.get_executable_path() {
                    // 验证自定义路径
                    if custom_path.exists() {
                        return Ok(custom_path);
                    } else {
                        return Err(format!(
                            "配置的自定义路径不存在: {}。请检查设置或使用默认路径。",
                            custom_path.display()
                        ));
                    }
                } else {
                    return Err("启用了自定义路径但未设置路径。请在设置中配置 Antigravity 可执行文件路径。".to_string());
                }
            }
        }
    }

    // 回退到默认查找逻辑
    get_antigravity_executable_path_default()
}

/// 获取 Antigravity 可执行文件路径（默认查找逻辑）
fn get_antigravity_executable_path_default() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let path = PathBuf::from("/Applications/Antigravity.app/Contents/MacOS/Antigravity");
        if path.exists() {
            return Ok(path);
        }
        Err("Antigravity 未在 /Applications 中找到。请在设置中配置自定义路径。".to_string())
    }

    #[cfg(target_os = "windows")]
    {
        use std::env;

        let local_appdata = env::var("LOCALAPPDATA").ok();
        let program_files = env::var("ProgramFiles")
            .unwrap_or_else(|_| "C:\\Program Files".to_string());

        let mut possible_paths = Vec::new();

        if let Some(local) = local_appdata {
            possible_paths.push(
                PathBuf::from(&local)
                    .join("Programs")
                    .join("Antigravity")
                    .join("Antigravity.exe")
            );
        }

        possible_paths.push(
            PathBuf::from(&program_files)
                .join("Antigravity")
                .join("Antigravity.exe")
        );

        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err("Antigravity 未找到。请在设置中配置自定义路径。".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        let possible_paths = vec![
            PathBuf::from("/usr/bin/antigravity"),
            PathBuf::from("/usr/local/bin/antigravity"),
            PathBuf::from("/opt/Antigravity/antigravity"),
        ];

        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err("Antigravity 未找到。请在设置中配置自定义路径。".to_string())
    }
}

/// 获取 Antigravity 可执行文件路径（保持向后兼容）
pub fn get_antigravity_executable_path() -> Result<PathBuf, String> {
    get_antigravity_executable_path_default()
}

/// 启动 Antigravity（支持自定义路径）
pub fn launch_antigravity_with_config(
    app_handle: Option<&tauri::AppHandle>
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // macOS 上优先尝试使用自定义路径
        if let Some(handle) = app_handle {
            if let Ok(config) = config::load_config(handle) {
                if config.use_custom_path {
                    if let Some(custom_path) = config.get_executable_path() {
                        if custom_path.exists() {
                            // 如果是 .app 包，使用 open 命令
                            if custom_path.to_string_lossy().contains(".app") {
                                Command::new("open")
                                    .arg("-a")
                                    .arg(&custom_path)
                                    .spawn()
                                    .map_err(|e| format!("启动 Antigravity 失败: {}", e))?;
                            } else {
                                // 否则直接执行
                                Command::new(&custom_path)
                                    .spawn()
                                    .map_err(|e| format!("启动 Antigravity 失败: {}", e))?;
                            }
                            return Ok(());
                        } else {
                            return Err(format!(
                                "配置的自定义路径不存在: {}",
                                custom_path.display()
                            ));
                        }
                    }
                }
            }
        }

        // 回退到默认路径
        let app_path = PathBuf::from("/Applications/Antigravity.app");
        if !app_path.exists() {
            return Err("Antigravity 未在 /Applications 中找到。请在设置中配置自定义路径。".to_string());
        }
        Command::new("open")
            .arg("-a")
            .arg(app_path)
            .spawn()
            .map_err(|e| format!("启动 Antigravity 失败: {}", e))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        let exe_path = get_antigravity_executable_path_with_config(app_handle)?;
        Command::new(exe_path)
            .spawn()
            .map_err(|e| format!("启动 Antigravity 失败: {}", e))?;
    }

    Ok(())
}

/// 启动 Antigravity（保持向后兼容）
pub fn launch_antigravity() -> Result<(), String> {
    launch_antigravity_with_config(None)
}
