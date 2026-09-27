use reposhelf_core::{discover, inspect, ScanOptions};
#[tauri::command]
fn scan(roots: Vec<String>) -> Result<Vec<reposhelf_core::RepoInfo>, String> { let paths=roots.into_iter().map(Into::into).collect::<Vec<_>>(); discover(&paths,&ScanOptions::default()).map_err(|e|e.to_string()).map(|paths| paths.into_iter().filter_map(|p|inspect(&p).ok()).collect()) }
#[tauri::command]
fn open_folder(path: String) -> Result<(),String> { reposhelf_platform::open_folder(path.as_ref()).map_err(|e|e.to_string()) }
fn main(){ tauri::Builder::default().invoke_handler(tauri::generate_handler![scan,open_folder]).run(tauri::generate_context!()).expect("failed to run RepoShelf") }
