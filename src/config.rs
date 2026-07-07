use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;
use chrono::Local;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    pub name: String,
    pub host: String,
    pub user: String,
    pub port: u16,
    pub key_name: Option<String>,
    pub environment: Option<String>,
    pub created_at: String,
    pub last_connected: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyInfo {
    pub name: String,
    pub key_type: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub server_name: String,
}

pub fn get_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("sshx")
}

pub fn get_keys_dir() -> PathBuf {
    get_config_dir().join("keys")
}

pub fn init_dirs() -> io::Result<()> {
    let config_dir = get_config_dir();
    let keys_dir = get_keys_dir();
    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&keys_dir)?;

    // Set correct permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700))?;
        fs::set_permissions(&keys_dir, fs::Permissions::from_mode(0o700))?;
    }

    Ok(())
}

pub fn load_servers() -> Vec<Server> {
    let path = get_config_dir().join("servers.yaml");
    if !path.exists() {
        return Vec::new();
    }
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    serde_yaml::from_reader(file).unwrap_or_default()
}

pub fn save_servers(servers: &[Server]) -> io::Result<()> {
    init_dirs()?;
    let path = get_config_dir().join("servers.yaml");
    let content = serde_yaml::to_string(servers)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(path, content)?;
    Ok(())
}

pub fn load_keys() -> Vec<KeyInfo> {
    let path = get_config_dir().join("keys.yaml");
    if !path.exists() {
        return Vec::new();
    }
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    serde_yaml::from_reader(file).unwrap_or_default()
}

pub fn save_keys(keys: &[KeyInfo]) -> io::Result<()> {
    init_dirs()?;
    let path = get_config_dir().join("keys.yaml");
    let content = serde_yaml::to_string(keys)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(path, content)?;
    Ok(())
}

pub fn add_history(server_name: &str) -> io::Result<()> {
    init_dirs()?;
    let path = get_config_dir().join("history.json");
    let mut history: Vec<HistoryEntry> = if path.exists() {
        let file = File::open(&path)?;
        serde_json::from_reader(file).unwrap_or_default()
    } else {
        Vec::new()
    };

    history.push(HistoryEntry {
        timestamp: Local::now().to_rfc3339(),
        server_name: server_name.to_string(),
    });

    // Keep history at a maximum of 1000 items
    if history.len() > 1000 {
        history.remove(0);
    }

    let content = serde_json::to_string_pretty(&history)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(path, content)?;
    Ok(())
}

// Struct to store Agent state in agent.env
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentEnv {
    pub socket: String,
    pub pid: u32,
}

pub fn save_agent_env(env: &AgentEnv) -> io::Result<()> {
    init_dirs()?;
    let path = get_config_dir().join("agent.env");
    let content = serde_json::to_string(env)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(path, content)?;
    Ok(())
}

pub fn load_agent_env() -> Option<AgentEnv> {
    let path = get_config_dir().join("agent.env");
    if !path.exists() {
        return None;
    }
    let file = File::open(path).ok()?;
    serde_json::from_reader(file).ok()
}

pub fn remove_agent_env() -> io::Result<()> {
    let path = get_config_dir().join("agent.env");
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn resolve_key(key_name: Option<&str>) -> Option<KeyInfo> {
    let keys = load_keys();
    let name_to_find = key_name.unwrap_or("default");
    keys.into_iter().find(|k| k.name == name_to_find)
}
