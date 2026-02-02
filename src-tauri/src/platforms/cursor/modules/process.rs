//! Cursor 进程管理模块

use std::process::Command;
use std::path::PathBuf;
use sysinfo::{System, ProcessRefreshKind, ProcessesToUpdate};

#[allow(dead_code)]
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

fn is_cursor_process(process: &sysinfo::Process) -> bool {
    let name = process.name().to_string_lossy().to_lowercase();
    if name.contains("cursor") && !name.contains("cursorfx") {
        return true;
    }

    let exe_path = process
        .exe()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_lowercase();

    #[cfg(target_os = "macos")]
    {
        return exe_path.contains("cursor.app");
    }

    #[cfg(target_os = "windows")]
    {
        return exe_path.ends_with("cursor.exe");
    }

    #[cfg(target_os = "linux")]
    {
        return exe_path.contains("/cursor");
    }
}

fn get_cursor_pids() -> Vec<u32> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::new());

    let mut pids = Vec::new();
    for (pid, process) in sys.processes() {
        if is_cursor_process(process) {
            pids.push(pid.as_u32());
        }
    }
    pids
}

/// 检查 Cursor 是否正在运行
pub fn is_cursor_running() -> bool {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::new());

    sys.processes().values().any(is_cursor_process)
}

/// 进程关闭结果
pub struct CloseResult {
    pub success: bool,
    pub warning: Option<String>,
}

/// 温和关闭 Cursor（带验证）
pub fn close_cursor(timeout_secs: u64) -> Result<(), String> {
    let result = close_cursor_with_result(timeout_secs);
    if !result.success {
        if let Some(warning) = result.warning {
            return Err(warning);
        }
    }
    Ok(())
}

/// 温和关闭 Cursor（返回详细结果）
pub fn close_cursor_with_result(timeout_secs: u64) -> CloseResult {
    #[cfg(target_os = "macos")]
    {
        let pids = get_cursor_pids();
        if pids.is_empty() {
            return CloseResult { success: true, warning: None };
        }

        // macOS: 尝试优雅退出
        let _ = Command::new("osascript")
            .args(["-e", "tell application \"Cursor\" to quit"])
            .output();

        // 等待优雅退出，最多 2 秒
        let quit_wait = std::cmp::min(timeout_secs, 2);
        let start_quit = std::time::Instant::now();
        while start_quit.elapsed() < std::time::Duration::from_secs(quit_wait) {
            if !is_cursor_running() {
                return CloseResult { success: true, warning: None };
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // 如果还在运行，发送 SIGTERM
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

        // 等待 SIGTERM 生效，最多 2 秒
        let graceful_timeout = std::cmp::min((timeout_secs * 7) / 10, 2);
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(graceful_timeout) {
            if !is_cursor_running() {
                return CloseResult { success: true, warning: None };
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // 强制杀死
        if is_cursor_running() {
            let remaining = get_cursor_pids();
            for pid in remaining {
                let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        // 最终验证
        if is_cursor_running() {
            return CloseResult {
                success: false,
                warning: Some("Cursor process still running after forced kill".to_string()),
            };
        }

        CloseResult { success: true, warning: None }
    }

    #[cfg(target_os = "windows")]
    {
        let pids = get_cursor_pids();
        if pids.is_empty() {
            return CloseResult { success: true, warning: None };
        }

        // Windows: 使用 taskkill 优雅关闭（带子进程树）
        let _ = Command::new("taskkill")
            .args(["/IM", "Cursor.exe", "/T"])
            .output();

        // 等待优雅退出，最多 2 秒
        let quit_wait = std::cmp::min(timeout_secs, 2);
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(quit_wait) {
            if !is_cursor_running() {
                return CloseResult { success: true, warning: None };
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        // 强制关闭
        if is_cursor_running() {
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/IM", "Cursor.exe"])
                .output();
            std::thread::sleep(std::time::Duration::from_millis(200));
        }

        // 最终验证
        if is_cursor_running() {
            return CloseResult {
                success: false,
                warning: Some("Cursor process still running after forced kill".to_string()),
            };
        }

        CloseResult { success: true, warning: None }
    }

    #[cfg(target_os = "linux")]
    {
        let _ = timeout_secs;
        let _ = kill_cursor_processes();

        // 最终验证
        std::thread::sleep(std::time::Duration::from_millis(200));
        if is_cursor_running() {
            return CloseResult {
                success: false,
                warning: Some("Cursor process still running after kill".to_string()),
            };
        }

        CloseResult { success: true, warning: None }
    }
}

/// 杀死所有 Cursor 进程
pub fn kill_cursor_processes() -> Result<(), String> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::new());

    let mut killed_count = 0;

    for (pid, process) in sys.processes() {
        if is_cursor_process(process) {
            if process.kill() {
                killed_count += 1;
                println!("Killed Cursor process: {} (PID: {})", process.name().to_string_lossy(), pid);
            }
        }
    }

    if killed_count > 0 {
        std::thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    } else {
        Err("No Cursor processes found".to_string())
    }
}

/// 获取 Cursor 可执行文件路径
pub fn get_cursor_executable_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "macos")]
    {
        let path = PathBuf::from("/Applications/Cursor.app/Contents/MacOS/Cursor");
        if path.exists() {
            return Ok(path);
        }
        Err("Cursor not found in /Applications".to_string())
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
                    .join("cursor")
                    .join("Cursor.exe")
            );
        }

        possible_paths.push(
            PathBuf::from(&program_files)
                .join("Cursor")
                .join("Cursor.exe")
        );

        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err("Cursor not found".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        let possible_paths = vec![
            PathBuf::from("/usr/bin/cursor"),
            PathBuf::from("/usr/local/bin/cursor"),
            PathBuf::from("/opt/Cursor/cursor"),
        ];

        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err("Cursor not found".to_string())
    }
}

/// 启动 Cursor（支持自定义路径）
pub fn launch_cursor_with_path(custom_path: Option<&str>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let app_path = if let Some(path) = custom_path {
            PathBuf::from(path)
        } else {
            PathBuf::from("/Applications/Cursor.app")
        };

        if !app_path.exists() {
            return Err(format!("Cursor not found at {:?}", app_path));
        }

        Command::new("open")
            .arg("-a")
            .arg(app_path)
            .spawn()
            .map_err(|e| format!("Failed to launch Cursor: {}", e))?;
    }

    #[cfg(target_os = "windows")]
    {
        let exe_path = if let Some(path) = custom_path {
            let p = PathBuf::from(path);
            if !p.exists() {
                return Err(format!("Cursor not found at {:?}", p));
            }
            p
        } else {
            get_cursor_executable_path()?
        };

        Command::new(exe_path)
            .spawn()
            .map_err(|e| format!("Failed to launch Cursor: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let exe_path = if let Some(path) = custom_path {
            let p = PathBuf::from(path);
            if !p.exists() {
                return Err(format!("Cursor not found at {:?}", p));
            }
            p
        } else {
            get_cursor_executable_path()?
        };

        Command::new(exe_path)
            .spawn()
            .map_err(|e| format!("Failed to launch Cursor: {}", e))?;
    }

    Ok(())
}

/// 启动 Cursor（使用默认路径）
pub fn launch_cursor() -> Result<(), String> {
    launch_cursor_with_path(None)
}

/// 验证 Cursor 路径是否有效
pub fn validate_cursor_path(path: &str) -> Result<bool, String> {
    let path_buf = PathBuf::from(path);

    if !path_buf.exists() {
        return Ok(false);
    }

    #[cfg(target_os = "windows")]
    {
        let file_name = path_buf.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        return Ok(file_name == "cursor.exe");
    }

    #[cfg(target_os = "macos")]
    {
        let path_str = path_buf.to_string_lossy().to_lowercase();
        return Ok(path_str.ends_with(".app") && path_str.contains("cursor"));
    }

    #[cfg(target_os = "linux")]
    {
        let file_name = path_buf.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        return Ok(file_name.contains("cursor"));
    }
}

