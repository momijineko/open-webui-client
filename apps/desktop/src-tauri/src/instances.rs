use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub url: String,
    pub mode: String,
}

#[tauri::command]
pub fn add_instance(_instance: Instance) -> Result<String, String> {
    Ok("Instance added".to_string())
}

#[tauri::command]
pub fn remove_instance(_id: String) -> Result<String, String> {
    Ok("Instance removed".to_string())
}

#[tauri::command]
pub fn get_instances() -> Result<Vec<Instance>, String> {
    Ok(vec![])
}

#[tauri::command]
pub fn connect_to_instance(_id: String) -> Result<String, String> {
    Ok("Connected".to_string())
}

#[tauri::command]
pub fn check_instance_status(_id: String) -> Result<String, String> {
    Ok("Running".to_string())
}
