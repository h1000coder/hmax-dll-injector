use std::{ffi::CString, os::windows::raw::HANDLE};

use rfd::FileDialog;
use serde::Serialize;
use sysinfo::System;
use windows::Win32::{Foundation::CloseHandle, Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation}, System::{Diagnostics::Debug::WriteProcessMemory, LibraryLoader::{GetModuleHandleA, GetProcAddress}, Memory::{MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx}, Threading::{CreateRemoteThread, GetCurrentProcess, LPTHREAD_START_ROUTINE, OpenProcess, OpenProcessToken, PROCESS_ALL_ACCESS}}};
use windows::core::s;


#[derive(Serialize)]
struct ProcessInfo {
    pid: u32,
    name: String,
}

#[tauri::command]
fn get_process_list() -> Vec<ProcessInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut processes: Vec<ProcessInfo> = sys.processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            name: process.name().to_str().expect("Erro ao transformar em string").to_string(),
            pid: pid.as_u32(),
        })
        .collect();

    processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    processes
}

#[tauri::command]
fn select_dll_file() -> String {
    let file = FileDialog::new()
        .add_filter("Dynamic Link Library", &["dll"])
        .set_title("Selecione a DLL para injetar")
        .pick_file();

    match file {
        Some(path) => path.display().to_string(),
        None => "".to_string(), // Retorna uma string vazia se o usuário cancelar a seleção
    }
}

#[tauri::command]
fn inject_dll(pid: u32, path: String) -> Result<String, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_ALL_ACCESS, false, pid)
            .map_err(|e| format!("Falha ao abrir o processo: {}", e))?;

        let path_cstr = CString::new(path).unwrap();
        let path_bytes = path_cstr.as_bytes_with_nul();

        let remote_mem = VirtualAllocEx(
            handle,
            None,
            path_bytes.len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
            );
        
        if remote_mem.is_null() {
            return Err("Falha ao alocar memoria no alvo".into());
        }

        WriteProcessMemory(
            handle,
            remote_mem,
            path_bytes.as_ptr() as _,
            path_bytes.len(),
            None,
        ).map_err(|e| format!("Falha ao escrever memoria: {}", e))?;
        
        let kernel32 = GetModuleHandleA(s!("kernel32.dll"))
            .map_err(|e| format!("Falha ao obter handle do kernel32: {}", e))?;

        let load_library_addr = GetProcAddress(kernel32, s!("LoadLibraryA"));

        let start_routine: LPTHREAD_START_ROUTINE = std::mem::transmute(load_library_addr);
        
        CreateRemoteThread(
            handle,
            None,
            0,
            start_routine,
            Some(remote_mem),
            0,
            None,
        ).map_err(|e| format!("Falha ao criar thread remota: {}", e))?;

        Ok("DLL Injetada com Sucesso!".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_process_list, select_dll_file, inject_dll])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
