#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proc {
    Dead,
    /// `started` is the creation time as a Windows FILETIME (100ns ticks since 1601), when readable.
    Alive { started: Option<u64> },
}

#[cfg(windows)]
pub fn probe(pid: u32) -> Proc {
    use windows_sys::Win32::Foundation::{
        CloseHandle, ERROR_ACCESS_DENIED, FILETIME, GetLastError, STILL_ACTIVE,
    };
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    if pid == 0 {
        return Proc::Dead;
    }
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            // Access denied means the process exists but belongs to someone more privileged.
            return if GetLastError() == ERROR_ACCESS_DENIED {
                Proc::Alive { started: None }
            } else {
                Proc::Dead
            };
        }
        // An exited process stays openable while anyone still holds a handle to it.
        let mut code = 0u32;
        let exited = GetExitCodeProcess(h, &mut code) != 0 && code != STILL_ACTIVE as u32;
        let zero = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let (mut created, mut exit, mut kernel, mut user) = (zero, zero, zero, zero);
        let started = if GetProcessTimes(h, &mut created, &mut exit, &mut kernel, &mut user) != 0 {
            Some(((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64)
        } else {
            None
        };
        CloseHandle(h);
        if exited { Proc::Dead } else { Proc::Alive { started } }
    }
}

#[cfg(not(windows))]
pub fn probe(_pid: u32) -> Proc {
    Proc::Alive { started: None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_process_is_alive() {
        match probe(std::process::id()) {
            Proc::Alive { started } => {
                if cfg!(windows) {
                    assert!(started.is_some_and(|t| t > 0));
                }
            }
            Proc::Dead => panic!("own process reported dead"),
        }
    }

    #[cfg(windows)]
    #[test]
    fn exited_process_is_dead() {
        let mut child = std::process::Command::new("cmd").args(["/C", "exit"]).spawn().unwrap();
        let pid = child.id();
        child.wait().unwrap();
        drop(child);
        assert_eq!(probe(pid), Proc::Dead);
    }

    #[cfg(windows)]
    #[test]
    fn pid_zero_is_dead() {
        assert_eq!(probe(0), Proc::Dead);
    }
}
