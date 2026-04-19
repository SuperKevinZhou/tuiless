use std::ffi::c_void;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::FromRawHandle;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::Mutex;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Console::{COORD, ClosePseudoConsole, CreatePseudoConsole, HPCON, ResizePseudoConsole};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT, InitializeProcThreadAttributeList,
    LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, STARTUPINFOEXW,
    UpdateProcThreadAttribute,
};
use windows::core::{PCWSTR, PWSTR};

pub struct PtySession {
    pub writer: Arc<Mutex<std::fs::File>>,
    pub reader: Arc<Mutex<std::fs::File>>,
    pub shell: String,
    hpc: HPCON,
    process_handle: HANDLE,
    thread_handle: HANDLE,
}

unsafe impl Send for PtySession {}
unsafe impl Sync for PtySession {}

impl PtySession {
    pub fn new(shell: &str, cwd: &Path, cols: u16, rows: u16) -> Result<Self> {
        unsafe {
            let mut input_read = HANDLE::default();
            let mut input_write = HANDLE::default();
            CreatePipe(&mut input_read, &mut input_write, None, 0)?;

            let mut output_read = HANDLE::default();
            let mut output_write = HANDLE::default();
            CreatePipe(&mut output_read, &mut output_write, None, 0)?;

            let size = COORD {
                X: cols as i16,
                Y: rows as i16,
            };
            let hpc = CreatePseudoConsole(size, input_read, output_write, 0)?;

            let mut attribute_list_size = 0usize;
            let _ = InitializeProcThreadAttributeList(None, 1, Some(0), &mut attribute_list_size);
            let mut attribute_list_storage = vec![0u8; attribute_list_size];
            let attribute_list = LPPROC_THREAD_ATTRIBUTE_LIST(attribute_list_storage.as_mut_ptr().cast());
            InitializeProcThreadAttributeList(Some(attribute_list), 1, Some(0), &mut attribute_list_size)?;

            UpdateProcThreadAttribute(
                attribute_list,
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
                Some((&hpc as *const HPCON).cast::<c_void>()),
                size_of::<HPCON>(),
                None,
                None,
            )?;

            let mut startup_info = STARTUPINFOEXW::default();
            startup_info.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
            startup_info.lpAttributeList = attribute_list;

            let mut commandline = to_utf16_mut(shell);
            let cwd_utf16 = to_utf16(cwd.as_os_str());
            let mut process_info = PROCESS_INFORMATION::default();
            CreateProcessW(
                PCWSTR::null(),
                Some(PWSTR(commandline.as_mut_ptr())),
                None,
                None,
                false,
                EXTENDED_STARTUPINFO_PRESENT,
                None,
                PCWSTR(cwd_utf16.as_ptr()),
                &startup_info.StartupInfo,
                &mut process_info,
            )
            .with_context(|| format!("failed to spawn shell `{shell}`"))?;

            DeleteProcThreadAttributeList(attribute_list);
            let _ = CloseHandle(input_read);
            let _ = CloseHandle(output_write);

            let writer = std::fs::File::from_raw_handle(input_write.0.cast());
            let reader = std::fs::File::from_raw_handle(output_read.0.cast());

            Ok(Self {
                writer: Arc::new(Mutex::new(writer)),
                reader: Arc::new(Mutex::new(reader)),
                shell: shell.to_string(),
                hpc,
                process_handle: process_info.hProcess,
                thread_handle: process_info.hThread,
            })
        }
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        unsafe {
            ResizePseudoConsole(
                self.hpc,
                COORD {
                    X: cols as i16,
                    Y: rows as i16,
                },
            )?;
        }
        Ok(())
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        unsafe {
            ClosePseudoConsole(self.hpc);
            let _ = CloseHandle(self.process_handle);
            let _ = CloseHandle(self.thread_handle);
        }
    }
}

pub fn default_shell() -> String {
    let candidates = ["pwsh.exe", "powershell.exe", "cmd.exe"];
    for candidate in candidates {
        if shell_exists(candidate) {
            return candidate.to_string();
        }
    }
    "cmd.exe".to_string()
}

fn shell_exists(name: &str) -> bool {
    std::process::Command::new("where")
        .arg(name)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn to_utf16(value: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    value.as_ref().encode_wide().chain(Some(0)).collect()
}

fn to_utf16_mut(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
