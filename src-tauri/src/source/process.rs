use std::collections::{HashMap, HashSet};

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

pub fn started(pid: u32) -> Option<u64> {
    match probe(pid) {
        Proc::Alive { started } => started,
        Proc::Dead => None,
    }
}

/// `root`'s descendants, breadth first. A child that started before its parent is a reused PID and is skipped.
pub fn descendants(parents: &HashMap<u32, u32>, root: u32, started: impl Fn(u32) -> Option<u64>) -> Vec<u32> {
    let mut out = Vec::new();
    let mut queue = vec![root];
    let mut seen = HashSet::from([root]);
    while let Some(p) = queue.pop() {
        let p_start = started(p);
        let mut kids: Vec<u32> = parents.iter().filter(|(c, pp)| **pp == p && **c != p).map(|(c, _)| *c).collect();
        kids.sort_unstable();
        for c in kids {
            if !seen.insert(c) {
                continue;
            }
            if let (Some(cs), Some(ps)) = (started(c), p_start) {
                if cs < ps {
                    continue;
                }
            }
            out.push(c);
            queue.insert(0, c);
        }
    }
    out
}

/// pid -> parent pid for every running process.
#[cfg(windows)]
pub fn parent_table() -> HashMap<u32, u32> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
    };
    let mut out = HashMap::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return out;
        }
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut ok = Process32FirstW(snap, &mut e);
        while ok != 0 {
            out.insert(e.th32ProcessID, e.th32ParentProcessID);
            ok = Process32NextW(snap, &mut e);
        }
        CloseHandle(snap);
    }
    out
}

#[cfg(not(windows))]
pub fn parent_table() -> HashMap<u32, u32> {
    HashMap::new()
}

/// Another process's command line (`ProcessCommandLineInformation`, Windows 8.1+).
#[cfg(windows)]
pub fn command_line(pid: u32) -> Option<String> {
    use windows_sys::Wdk::System::Threading::NtQueryInformationProcess;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    #[repr(C)]
    struct UnicodeString {
        length: u16,
        maximum_length: u16,
        buffer: *const u16,
    }
    const PROCESS_COMMAND_LINE_INFORMATION: i32 = 60;

    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return None;
        }
        let mut len = 0u32;
        NtQueryInformationProcess(h, PROCESS_COMMAND_LINE_INFORMATION, std::ptr::null_mut(), 0, &mut len);
        let mut out = None;
        if len as usize >= std::mem::size_of::<UnicodeString>() {
            // u64 backing keeps the struct at the buffer start aligned.
            let mut buf = vec![0u64; (len as usize).div_ceil(8)];
            if NtQueryInformationProcess(h, PROCESS_COMMAND_LINE_INFORMATION, buf.as_mut_ptr().cast(), len, &mut len) >= 0 {
                let us = &*(buf.as_ptr() as *const UnicodeString);
                if !us.buffer.is_null() {
                    let chars = std::slice::from_raw_parts(us.buffer, us.length as usize / 2);
                    out = Some(String::from_utf16_lossy(chars));
                }
            }
        }
        CloseHandle(h);
        out
    }
}

#[cfg(not(windows))]
pub fn command_line(_pid: u32) -> Option<String> {
    None
}

/// (owning pid, port) for every listening TCP socket, IPv4 and IPv6.
#[cfg(windows)]
pub fn listening_ports() -> Vec<(u32, u16)> {
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetExtendedTcpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCPROW_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER,
    };
    const AF_INET: u32 = 2;
    const AF_INET6: u32 = 23;

    fn table(af: u32) -> Vec<u64> {
        let mut size = 0u32;
        for _ in 0..4 {
            let mut buf = vec![0u64; (size as usize).div_ceil(8).max(1)];
            let mut got = (buf.len() * 8) as u32;
            let rc = unsafe { GetExtendedTcpTable(buf.as_mut_ptr().cast(), &mut got, 0, af, TCP_TABLE_OWNER_PID_LISTENER, 0) };
            if rc == 0 {
                return buf;
            }
            size = got + 1024;
        }
        Vec::new()
    }
    fn rows<R>(buf: &[u64]) -> &[R] {
        if buf.is_empty() {
            return &[];
        }
        unsafe {
            let n = *(buf.as_ptr() as *const u32) as usize;
            // The row array follows dwNumEntries at the row type's alignment.
            let first = (buf.as_ptr() as *const u8).add(std::mem::align_of::<R>().max(4)) as *const R;
            std::slice::from_raw_parts(first, n)
        }
    }

    let mut out = Vec::new();
    let v4 = table(AF_INET);
    out.extend(rows::<MIB_TCPROW_OWNER_PID>(&v4).iter().map(|r| (r.dwOwningPid, u16::from_be(r.dwLocalPort as u16))));
    let v6 = table(AF_INET6);
    out.extend(rows::<MIB_TCP6ROW_OWNER_PID>(&v6).iter().map(|r| (r.dwOwningPid, u16::from_be(r.dwLocalPort as u16))));
    out
}

#[cfg(not(windows))]
pub fn listening_ports() -> Vec<(u32, u16)> {
    Vec::new()
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

    #[test]
    fn descendants_walk_the_tree_and_skip_reused_pids() {
        let parents = HashMap::from([(2, 1), (3, 2), (4, 2), (5, 9), (6, 1), (1, 1)]);
        let started = |pid: u32| Some(if pid == 6 { 0 } else { pid as u64 * 10 });
        assert_eq!(descendants(&parents, 1, started), vec![2, 3, 4]);
        assert_eq!(descendants(&parents, 1, |_| None), vec![2, 6, 3, 4]);
        assert!(descendants(&parents, 7, started).is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn real_child_is_found_with_its_command_line() {
        let mut child = std::process::Command::new("cmd").args(["/C", "ping -n 3 127.0.0.1 >nul"]).spawn().unwrap();
        let me = std::process::id();
        let kids = descendants(&parent_table(), me, started);
        assert!(kids.contains(&child.id()), "{kids:?}");
        let cmd = command_line(child.id()).unwrap_or_default();
        assert!(cmd.contains("ping -n 3 127.0.0.1"), "{cmd}");
        let _ = child.kill();
        let _ = child.wait();
    }

    #[cfg(windows)]
    #[test]
    fn own_listening_sockets_are_listed_with_their_ports() {
        let v4 = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let v6 = std::net::TcpListener::bind("[::1]:0").ok();
        let ports = listening_ports();
        let me = std::process::id();
        assert!(ports.contains(&(me, v4.local_addr().unwrap().port())), "{ports:?}");
        if let Some(v6) = v6 {
            assert!(ports.contains(&(me, v6.local_addr().unwrap().port())));
        }
    }
}
