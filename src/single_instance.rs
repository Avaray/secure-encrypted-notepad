/// Single Instance Mode — prevents multiple SEN processes and forwards file paths via IPC.
///
/// - Windows: Named Mutex for locking, Named Pipe for IPC.
/// - Linux: Lock file with flock, Unix domain socket for IPC.
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Result of attempting to acquire the single-instance lock.
pub enum LockResult {
    /// We are the primary instance. Contains the IPC file queue.
    Acquired(Arc<Mutex<Vec<PathBuf>>>),
    /// Another instance is already running; we sent the file path and should exit.
    AlreadyRunning,
}

// ============================================================================
// Windows implementation
// ============================================================================
#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::io::{BufRead, BufReader};

    const MUTEX_NAME: &str = "Global\\SEN_SingleInstance_Mutex";
    const PIPE_NAME: &str = r"\\.\pipe\sen_single_instance";

    /// Attempt to acquire the single-instance lock.
    /// If acquired, starts an IPC listener thread and returns the file queue.
    pub fn try_lock(file_to_open: &Option<PathBuf>) -> LockResult {
        use windows_sys::Win32::Foundation::*;

        let mutex_name: Vec<u16> = MUTEX_NAME
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let handle = unsafe {
            windows_sys::Win32::System::Threading::CreateMutexW(
                std::ptr::null(),
                0,
                mutex_name.as_ptr(),
            )
        };

        if handle.is_null() {
            // Failed to create mutex — assume another instance
            if let Some(path) = file_to_open {
                let _ = send_path(path);
            }
            bring_to_foreground();
            return LockResult::AlreadyRunning;
        }

        let last_error = unsafe { GetLastError() };
        if last_error == ERROR_ALREADY_EXISTS {
            // Another instance owns the mutex
            unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
            if let Some(path) = file_to_open {
                let _ = send_path(path);
            }
            bring_to_foreground();
            return LockResult::AlreadyRunning;
        }

        // We are the primary instance — start IPC listener
        let queue = Arc::new(Mutex::new(Vec::<PathBuf>::new()));
        let queue_clone = Arc::clone(&queue);

        std::thread::Builder::new()
            .name("ipc-listener".to_string())
            .spawn(move || {
                listen_for_connections(queue_clone);
            })
            .expect("Failed to spawn IPC listener thread");

        LockResult::Acquired(queue)
    }

    fn listen_for_connections(queue: Arc<Mutex<Vec<PathBuf>>>) {
        use std::os::windows::io::FromRawHandle;

        loop {
            let pipe_name: Vec<u16> = PIPE_NAME.encode_utf16().chain(std::iter::once(0)).collect();

            let handle = unsafe {
                windows_sys::Win32::System::Pipes::CreateNamedPipeW(
                    pipe_name.as_ptr(),
                    0x00000003,              // PIPE_ACCESS_DUPLEX
                    0x00000004 | 0x00000002, // PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE
                    1,                       // max instances
                    4096,                    // out buffer
                    4096,                    // in buffer
                    0,                       // default timeout
                    std::ptr::null(),
                )
            };

            if handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
                crate::sen_debug!("IPC: Failed to create named pipe");
                std::thread::sleep(std::time::Duration::from_secs(1));
                continue;
            }

            // Wait for a client to connect
            let connected = unsafe {
                windows_sys::Win32::System::Pipes::ConnectNamedPipe(handle, std::ptr::null_mut())
            };

            if connected == 0 {
                let err = unsafe { windows_sys::Win32::Foundation::GetLastError() };
                if err != 535 {
                    // ERROR_PIPE_CONNECTED = 535 is OK (client connected before we called ConnectNamedPipe)
                    unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
                    continue;
                }
            }

            // Read the path from the pipe
            let file = unsafe { std::fs::File::from_raw_handle(handle as *mut std::ffi::c_void) };
            let reader = BufReader::new(&file);
            for line in reader.lines() {
                if let Ok(path_str) = line {
                    let path_str = path_str.trim().to_string();
                    if !path_str.is_empty() {
                        crate::sen_debug!("IPC: Received path: {}", path_str);
                        if let Ok(mut q) = queue.lock() {
                            q.push(PathBuf::from(path_str));
                        }
                    }
                    break;
                }
            }
            // File handle is dropped here, closing the pipe end
        }
    }

    fn send_path(path: &PathBuf) -> std::io::Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        // Try to connect to the named pipe
        let mut attempts = 0;
        loop {
            match OpenOptions::new().read(true).write(true).open(PIPE_NAME) {
                Ok(mut file) => {
                    let path_str = format!("{}\n", path.display());
                    file.write_all(path_str.as_bytes())?;
                    file.flush()?;
                    crate::sen_debug!("IPC: Sent path to existing instance: {}", path.display());
                    return Ok(());
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= 5 {
                        crate::sen_debug!(
                            "IPC: Failed to connect to pipe after {} attempts: {}",
                            attempts,
                            e
                        );
                        return Err(e);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }

    fn bring_to_foreground() {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;

        let window_title: Vec<u16> = "Secure Encrypted Notepad\0".encode_utf16().collect();

        unsafe {
            let hwnd = FindWindowW(std::ptr::null(), window_title.as_ptr());
            if !hwnd.is_null() {
                if IsIconic(hwnd) != 0 {
                    ShowWindow(hwnd, 9); // SW_RESTORE
                }
                SetForegroundWindow(hwnd);
                crate::sen_debug!("IPC: Brought existing window to foreground");
            }
        }
    }
}

// ============================================================================
// Linux implementation
// ============================================================================
#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::{UnixListener, UnixStream};

    const SOCKET_PATH: &str = "/tmp/sen_single_instance.sock";
    const LOCK_PATH: &str = "/tmp/sen_single_instance.lock";

    pub fn try_lock(file_to_open: &Option<PathBuf>) -> LockResult {
        use std::fs::OpenOptions;
        use std::os::unix::io::AsRawFd;

        // Try to acquire an exclusive flock
        let lock_file = OpenOptions::new().create(true).write(true).open(LOCK_PATH);

        let lock_file = match lock_file {
            Ok(f) => f,
            Err(_) => {
                if let Some(path) = file_to_open {
                    let _ = send_path(path);
                }
                return LockResult::AlreadyRunning;
            }
        };

        let fd = lock_file.as_raw_fd();
        let result = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };

        if result != 0 {
            // Another instance holds the lock
            if let Some(path) = file_to_open {
                let _ = send_path(path);
            }
            return LockResult::AlreadyRunning;
        }

        // We are the primary instance — remove stale socket and start listener
        let _ = std::fs::remove_file(SOCKET_PATH);

        let listener = match UnixListener::bind(SOCKET_PATH) {
            Ok(l) => l,
            Err(e) => {
                crate::sen_debug!("IPC: Failed to bind Unix socket: {}", e);
                let queue = Arc::new(Mutex::new(Vec::<PathBuf>::new()));
                return LockResult::Acquired(queue);
            }
        };

        let queue = Arc::new(Mutex::new(Vec::<PathBuf>::new()));
        let queue_clone = Arc::clone(&queue);

        std::thread::Builder::new()
            .name("ipc-listener".to_string())
            .spawn(move || {
                let _lock_file = lock_file; // Keep lock file alive
                for stream in listener.incoming() {
                    if let Ok(stream) = stream {
                        let reader = BufReader::new(&stream);
                        for line in reader.lines() {
                            if let Ok(path_str) = line {
                                let path_str = path_str.trim().to_string();
                                if !path_str.is_empty() {
                                    crate::sen_debug!("IPC: Received path: {}", path_str);
                                    if let Ok(mut q) = queue_clone.lock() {
                                        q.push(PathBuf::from(path_str));
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            })
            .expect("Failed to spawn IPC listener thread");

        LockResult::Acquired(queue)
    }

    fn send_path(path: &PathBuf) -> std::io::Result<()> {
        let mut stream = UnixStream::connect(SOCKET_PATH)?;
        let path_str = format!("{}\n", path.display());
        stream.write_all(path_str.as_bytes())?;
        stream.flush()?;
        crate::sen_debug!("IPC: Sent path to existing instance: {}", path.display());
        Ok(())
    }
}

// ============================================================================
// Public API (re-exports platform impl)
// ============================================================================

/// Attempt to acquire the single-instance lock.
/// Returns `LockResult::Acquired` with the IPC queue if we are the primary instance,
/// or `LockResult::AlreadyRunning` if another instance is already running (file path was forwarded).
pub fn try_lock(file_to_open: &Option<PathBuf>) -> LockResult {
    platform::try_lock(file_to_open)
}
