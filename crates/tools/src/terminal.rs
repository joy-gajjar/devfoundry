//! Native PTY capability boundary. This is intentionally separate from the
//! legacy pipe-backed `PtyTool`.

use crate::{ToolContext, authorize};
use std::{path::Path, sync::Arc, time::Duration};
use tokio::sync::{Mutex, Notify};

pub const MAX_PTY_INPUT_BYTES: usize = 64 * 1024;
pub const MAX_PTY_OUTPUT_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PtyCapability {
    pub native: bool,
    pub platform: &'static str,
}

#[derive(Clone, Debug)]
pub struct PtyOpenRequest {
    pub program: String,
    pub arguments: Vec<String>,
    pub cwd: Option<String>,
    pub rows: u16,
    pub columns: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputLease(u64);

impl InputLease {
    pub fn next_generation(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

#[derive(Clone, Debug)]
pub struct PtyInput {
    lease: InputLease,
    bytes: Vec<u8>,
}

impl PtyInput {
    pub fn new(lease: InputLease, bytes: Vec<u8>) -> Self {
        Self { lease, bytes }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PtyResize {
    pub rows: u16,
    pub columns: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PtyOutput {
    pub offset: u64,
    pub next_offset: u64,
    pub bytes: Vec<u8>,
    pub truncated: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum NativePtyError {
    #[error("native PTY is unsupported on this platform")]
    UnsupportedPlatform,
    #[error("permission denied for native PTY session")]
    PermissionDenied,
    #[error("invalid native PTY request: {0}")]
    InvalidRequest(String),
    #[error("native PTY input lease is invalid")]
    InvalidInputLease,
    #[error("native PTY input is too large")]
    InputTooLarge,
    #[error("native PTY output offset is no longer available")]
    OutputGap,
    #[error("native PTY produced no output before the deadline")]
    NoOutput,
    #[error("native PTY failed: {0}")]
    Io(String),
    #[error("native PTY operation was cancelled")]
    Cancelled,
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::{
        ffi::CString,
        io::{Read, Write},
        os::fd::{AsRawFd, FromRawFd, RawFd},
        thread,
    };

    struct State {
        buffer: Vec<u8>,
        base: u64,
        closed: bool,
    }

    pub struct NativePtyService {
        master: Arc<Mutex<std::fs::File>>,
        state: Arc<Mutex<State>>,
        notify: Arc<Notify>,
        pid: libc::pid_t,
        lease: InputLease,
        closed: Arc<Mutex<bool>>,
        cancellation: tokio_util::sync::CancellationToken,
    }

    impl NativePtyService {
        pub fn capability() -> PtyCapability {
            PtyCapability {
                native: true,
                platform: "unix",
            }
        }

        pub async fn open(
            request: PtyOpenRequest,
            context: ToolContext,
        ) -> Result<Self, NativePtyError> {
            validate_request(&request)?;
            if context.cancellation.is_cancelled() {
                return Err(NativePtyError::Cancelled);
            }
            authorize(&context, "pty_session", &request.program)
                .await
                .map_err(|error| match error {
                    crate::ToolError::Domain(
                        devfoundry_schema::DomainError::PermissionDenied { .. },
                    ) => NativePtyError::PermissionDenied,
                    other => NativePtyError::Io(other.to_string()),
                })?;
            let cwd = match request.cwd.as_deref() {
                Some(path) => context
                    .resolve(path)
                    .map_err(|error| NativePtyError::Io(error.to_string()))?,
                None => context
                    .root
                    .canonicalize()
                    .map_err(|error| NativePtyError::Io(error.to_string()))?,
            };
            if !cwd.is_dir() {
                return Err(NativePtyError::InvalidRequest(
                    "cwd is not a directory".into(),
                ));
            }
            let (master_fd, pid) = spawn(&request, &cwd)?;
            resize_fd(master_fd, request.rows, request.columns)?;
            let master = Arc::new(Mutex::new(unsafe { std::fs::File::from_raw_fd(master_fd) }));
            let state = Arc::new(Mutex::new(State {
                buffer: Vec::new(),
                base: 0,
                closed: false,
            }));
            let notify = Arc::new(Notify::new());
            start_reader(Arc::clone(&master), Arc::clone(&state), Arc::clone(&notify));
            let closed = Arc::new(Mutex::new(false));
            Ok(Self {
                master,
                state,
                notify,
                pid,
                lease: InputLease(1),
                closed,
                cancellation: context.cancellation,
            })
        }

        pub fn input_lease(&self) -> InputLease {
            self.lease
        }

        pub async fn input(&self, input: PtyInput) -> Result<(), NativePtyError> {
            if self.cancellation.is_cancelled() || *self.closed.lock().await {
                return Err(NativePtyError::Cancelled);
            }
            if input.lease != self.lease {
                return Err(NativePtyError::InvalidInputLease);
            }
            if input.bytes.len() > MAX_PTY_INPUT_BYTES {
                return Err(NativePtyError::InputTooLarge);
            }
            let mut master = self.master.lock().await;
            master
                .write_all(&input.bytes)
                .map_err(|error| NativePtyError::Io(error.to_string()))
        }

        pub async fn resize(&self, resize: PtyResize) -> Result<(), NativePtyError> {
            if resize.rows == 0 || resize.columns == 0 {
                return Err(NativePtyError::InvalidRequest(
                    "terminal dimensions must be nonzero".into(),
                ));
            }
            let master = self.master.lock().await;
            resize_fd(master.as_raw_fd(), resize.rows, resize.columns)
        }

        pub async fn read_output(
            &self,
            offset: u64,
            timeout: Duration,
        ) -> Result<PtyOutput, NativePtyError> {
            let deadline = tokio::time::sleep(timeout);
            tokio::pin!(deadline);
            loop {
                let state = self.state.lock().await;
                if offset < state.base {
                    return Err(NativePtyError::OutputGap);
                }
                let end = state.base + state.buffer.len() as u64;
                if offset < end {
                    let start = (offset - state.base) as usize;
                    return Ok(PtyOutput {
                        offset,
                        next_offset: end,
                        bytes: state.buffer[start..].to_vec(),
                        truncated: state.base > 0,
                    });
                }
                if state.closed {
                    return Err(NativePtyError::NoOutput);
                }
                drop(state);
                tokio::select! { _ = self.notify.notified() => {}, _ = &mut deadline => return Err(NativePtyError::NoOutput) }
            }
        }

        pub async fn close(&self) -> Result<(), NativePtyError> {
            let mut closed = self.closed.lock().await;
            if *closed {
                return Ok(());
            }
            *closed = true;
            terminate_pid(self.pid);
            let mut status = 0;
            let result = unsafe { libc::waitpid(self.pid, &mut status, 0) };
            if result < 0 {
                return Err(NativePtyError::Io(
                    std::io::Error::last_os_error().to_string(),
                ));
            }
            let mut state = self.state.lock().await;
            state.closed = true;
            self.notify.notify_waiters();
            Ok(())
        }
    }

    impl Drop for NativePtyService {
        fn drop(&mut self) {
            if let Ok(closed) = self.closed.try_lock() {
                if !*closed {
                    terminate_pid(self.pid);
                    unsafe { libc::waitpid(self.pid, std::ptr::null_mut(), 0) };
                }
            }
        }
    }

    fn validate_request(request: &PtyOpenRequest) -> Result<(), NativePtyError> {
        if request.program.is_empty() || request.program.as_bytes().contains(&0) {
            return Err(NativePtyError::InvalidRequest("program is invalid".into()));
        }
        if request.rows == 0 || request.columns == 0 {
            return Err(NativePtyError::InvalidRequest(
                "terminal dimensions must be nonzero".into(),
            ));
        }
        if request.arguments.len() > 128
            || request
                .arguments
                .iter()
                .any(|arg| arg.as_bytes().contains(&0))
        {
            return Err(NativePtyError::InvalidRequest(
                "arguments are invalid".into(),
            ));
        }
        Ok(())
    }

    fn spawn(request: &PtyOpenRequest, cwd: &Path) -> Result<(RawFd, libc::pid_t), NativePtyError> {
        let program = CString::new(request.program.as_str())
            .map_err(|_| NativePtyError::InvalidRequest("program contains NUL".into()))?;
        let args = request
            .arguments
            .iter()
            .map(|arg| CString::new(arg.as_str()).unwrap())
            .collect::<Vec<_>>();
        let mut argv = vec![program.as_ptr()];
        argv.extend(args.iter().map(|arg| arg.as_ptr()));
        argv.push(std::ptr::null());
        let cwd = CString::new(cwd.to_string_lossy().as_bytes())
            .map_err(|_| NativePtyError::InvalidRequest("cwd contains NUL".into()))?;
        let mut master = -1;
        let pid = unsafe {
            libc::forkpty(
                &mut master,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if pid < 0 {
            return Err(NativePtyError::Io(
                std::io::Error::last_os_error().to_string(),
            ));
        }
        if pid == 0 {
            unsafe {
                if libc::chdir(cwd.as_ptr()) != 0 {
                    libc::_exit(126);
                }
                let environment = [std::ptr::null()];
                libc::execve(program.as_ptr(), argv.as_ptr(), environment.as_ptr());
                libc::_exit(127);
            }
        }
        Ok((master, pid))
    }

    fn resize_fd(fd: RawFd, rows: u16, columns: u16) -> Result<(), NativePtyError> {
        let size = libc::winsize {
            ws_row: rows,
            ws_col: columns,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let result = unsafe { libc::ioctl(fd, libc::TIOCSWINSZ, &size) };
        if result == -1 {
            Err(NativePtyError::Io(
                std::io::Error::last_os_error().to_string(),
            ))
        } else {
            Ok(())
        }
    }

    fn terminate_pid(pid: libc::pid_t) {
        unsafe {
            if libc::kill(-pid, libc::SIGKILL) != 0 {
                libc::kill(pid, libc::SIGKILL);
            }
        }
    }

    fn start_reader(
        master: Arc<Mutex<std::fs::File>>,
        state: Arc<Mutex<State>>,
        notify: Arc<Notify>,
    ) {
        thread::spawn(move || {
            let mut buffer = [0_u8; 8192];
            loop {
                let count = {
                    let mut file = master.blocking_lock();
                    file.read(&mut buffer)
                };
                match count {
                    Ok(0) | Err(_) => {
                        state.blocking_lock().closed = true;
                        notify.notify_waiters();
                        break;
                    }
                    Ok(count) => {
                        let mut current = state.blocking_lock();
                        current.buffer.extend_from_slice(&buffer[..count]);
                        if current.buffer.len() > MAX_PTY_OUTPUT_BYTES {
                            let excess = current.buffer.len() - MAX_PTY_OUTPUT_BYTES;
                            current.buffer.drain(..excess);
                            current.base += excess as u64;
                        }
                        notify.notify_waiters();
                    }
                }
            }
        });
    }
}

#[cfg(unix)]
pub use unix::NativePtyService;

#[cfg(not(unix))]
pub struct NativePtyService;

#[cfg(not(unix))]
impl NativePtyService {
    pub fn capability() -> PtyCapability {
        PtyCapability {
            native: false,
            platform: "unsupported",
        }
    }
    pub async fn open(_: PtyOpenRequest, _: ToolContext) -> Result<Self, NativePtyError> {
        Err(NativePtyError::UnsupportedPlatform)
    }
    pub fn input_lease(&self) -> InputLease {
        InputLease(0)
    }
    pub async fn input(&self, _: PtyInput) -> Result<(), NativePtyError> {
        Err(NativePtyError::UnsupportedPlatform)
    }
    pub async fn resize(&self, _: PtyResize) -> Result<(), NativePtyError> {
        Err(NativePtyError::UnsupportedPlatform)
    }
    pub async fn read_output(&self, _: u64, _: Duration) -> Result<PtyOutput, NativePtyError> {
        Err(NativePtyError::UnsupportedPlatform)
    }
    pub async fn close(&self) -> Result<(), NativePtyError> {
        Err(NativePtyError::UnsupportedPlatform)
    }
}
