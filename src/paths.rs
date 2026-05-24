use std::env;
use std::path::{Path, PathBuf};

/// Get the Omara workspace directory.
/// Priority:
/// 1. Environment variable OMARA_WORKSPACE_DIR
/// 2. Fallback: ~/Projects/omara-labs
pub fn get_workspace_dir() -> PathBuf {
    if let Ok(env_dir) = env::var("OMARA_WORKSPACE_DIR") {
        return PathBuf::from(env_dir);
    }
    
    // Default fallback to ~/Projects/omara-labs
    let home = env::var("HOME").unwrap_or_else(|_| "/home/jeryd".to_string());
    Path::new(&home).join("Projects").join("omara-labs")
}

/// Helper to get a subcomponent path relative to the workspace.
/// E.g. get_component_path("omara-apps") -> ~/Projects/omara-labs/omara-apps
pub fn get_component_path(component: &str) -> PathBuf {
    // Check if there is a specific env override for the component, e.g. OMARA_APPS_DIR
    let env_key = format!("OMARA_{}_DIR", component.to_uppercase().replace("-", "_"));
    if let Ok(env_dir) = env::var(&env_key) {
        return PathBuf::from(env_dir);
    }
    
    // If the component is "omara-core", it was renamed to "omara-configs"
    let folder_name = if component == "omara-core" {
        "omara-configs"
    } else {
        component
    };
    
    get_workspace_dir().join(folder_name)
}
