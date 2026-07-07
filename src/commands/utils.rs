use std::process::Command;
use std::io::{self, Write};
use std::fs;
use std::path::PathBuf;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use crate::config::{
    load_servers, save_servers, Server, load_keys, save_keys, KeyInfo, get_config_dir
};
use crate::commands::agent::setup_agent_env;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct ExportData {
    servers: Vec<Server>,
    keys: Vec<KeyInfo>,
}

pub fn copy_files(src: &str, dest: &str) -> io::Result<()> {
    let servers = load_servers();
    let keys = load_keys();

    // Parse src
    let (real_src, src_port, src_key) = parse_scp_arg(src, &servers, &keys);
    // Parse dest
    let (real_dest, dest_port, dest_key) = parse_scp_arg(dest, &servers, &keys);

    let mut cmd = Command::new("scp");
    setup_agent_env(&mut cmd);

    // Use port if resolved from either arg
    let port = src_port.or(dest_port);
    if let Some(p) = port {
        cmd.arg("-P").arg(p.to_string());
    }

    // Use key if resolved from either arg
    let key = src_key.or(dest_key);
    if let Some(k) = key {
        cmd.arg("-i").arg(k);
    }

    cmd.arg(&real_src);
    cmd.arg(&real_dest);

    println!("Executando: scp...");
    let status = cmd.status()?;
    if status.success() {
        println!("Cópia de arquivos concluída com sucesso.");
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Comando scp retornou um erro."))
    }
}

fn parse_scp_arg(arg: &str, servers: &[Server], keys: &[KeyInfo]) -> (String, Option<u16>, Option<String>) {
    if let Some(pos) = arg.find(':') {
        let server_name = &arg[..pos];
        let path = &arg[pos+1..];
        if let Some(s) = servers.iter().find(|s| s.name == server_name) {
            let key_path = s.key_name.as_ref().and_then(|kn| {
                keys.iter().find(|k| &k.name == kn).map(|k| k.path.clone())
            });
            return (format!("{}@{}:{}", s.user, s.host, path), Some(s.port), key_path);
        }
    }
    (arg.to_string(), None, None)
}

pub fn exec_command(server_name: &str, remote_cmd: &str) -> io::Result<()> {
    let servers = load_servers();
    let server = servers.iter().find(|s| s.name == server_name)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Servidor '{}' não encontrado.", server_name)
        ))?;

    let mut cmd = Command::new("ssh");
    setup_agent_env(&mut cmd);

    if let Some(ref k_name) = server.key_name {
        let keys = load_keys();
        if let Some(k) = keys.iter().find(|k| &k.name == k_name) {
            cmd.arg("-i").arg(&k.path);
        }
    }

    cmd.args(&[
        "-p", &server.port.to_string(),
        &format!("{}@{}", server.user, server.host),
        remote_cmd
    ]);

    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Comando remoto retornou falha."))
    }
}

pub fn ping_server(server_name: &str) -> io::Result<()> {
    let servers = load_servers();
    let server = servers.iter().find(|s| s.name == server_name)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Servidor '{}' não encontrado.", server_name)
        ))?;

    println!("Pingando servidor '{}' ({}@{} -p {})...", server.name, server.user, server.host, server.port);

    let mut cmd = Command::new("ssh");
    setup_agent_env(&mut cmd);

    if let Some(ref k_name) = server.key_name {
        let keys = load_keys();
        if let Some(k) = keys.iter().find(|k| &k.name == k_name) {
            cmd.arg("-i").arg(&k.path);
        }
    }

    // Connect with timeout and exit immediately
    cmd.args(&[
        "-o", "ConnectTimeout=5",
        "-o", "BatchMode=yes",
        "-p", &server.port.to_string(),
        &format!("{}@{}", server.user, server.host),
        "exit"
    ]);

    let status = cmd.status()?;
    if status.success() {
        println!("\x1b[32mConexão estabelecida com sucesso. Servidor '{}' está ONLINE.\x1b[0m", server_name);
        Ok(())
    } else {
        println!("\x1b[31mFalha na conexão. Servidor '{}' está OFFLINE ou INACESSÍVEL.\x1b[0m", server_name);
        Err(io::Error::new(io::ErrorKind::ConnectionRefused, "Falha na conexão SSH."))
    }
}

pub fn show_config(server_name: &str) -> io::Result<()> {
    let servers = load_servers();
    let server = servers.iter().find(|s| s.name == server_name)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Servidor '{}' não encontrado.", server_name)
        ))?;

    println!("Host {}", server.name);
    println!("    HostName {}", server.host);
    println!("    User {}", server.user);
    println!("    Port {}", server.port);
    if let Some(ref k_name) = server.key_name {
        let keys = load_keys();
        if let Some(k) = keys.iter().find(|k| &k.name == k_name) {
            println!("    IdentityFile {}", k.path);
        }
    }
    Ok(())
}

pub fn export_config(format: &str) -> io::Result<()> {
    let data = ExportData {
        servers: load_servers(),
        keys: load_keys(),
    };

    let output = match format.to_lowercase().as_str() {
        "json" => serde_json::to_string_pretty(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
        "toml" => toml::to_string_pretty(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
        _ => serde_yaml::to_string(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?,
    };

    println!("{}", output);
    Ok(())
}

pub fn import_config(filepath: &str) -> io::Result<()> {
    let path = PathBuf::from(filepath);
    if !path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Arquivo '{}' não encontrado.", filepath)));
    }

    let content = fs::read_to_string(&path)?;
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("yaml")
        .to_lowercase();

    let data: ExportData = match ext.as_str() {
        "json" => serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        "toml" => toml::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        _ => serde_yaml::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
    };

    // Save servers
    let mut servers = load_servers();
    for imported_s in data.servers {
        if let Some(pos) = servers.iter().position(|s| s.name == imported_s.name) {
            servers[pos] = imported_s;
        } else {
            servers.push(imported_s);
        }
    }
    save_servers(&servers)?;

    // Save keys
    let mut keys = load_keys();
    for imported_k in data.keys {
        if let Some(pos) = keys.iter().position(|k| k.name == imported_k.name) {
            keys[pos] = imported_k;
        } else {
            keys.push(imported_k);
        }
    }
    save_keys(&keys)?;

    println!("Configuração importada com sucesso de '{}'.", filepath);
    Ok(())
}

pub fn run_doctor() -> io::Result<()> {
    println!("=== SSHX Doctor - Diagnóstico do Sistema ===");
    let mut issues = 0;

    let config_dir = get_config_dir();
    println!("Diretório de Configuração: {}", config_dir.display());

    // 1. Check folder permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(&config_dir) {
            let mode = metadata.permissions().mode() & 0o777;
            if mode != 0o700 {
                println!("\x1b[31m[ERRO] Permissões incorretas no diretório de configuração: {:o} (deveria ser 700)\x1b[0m", mode);
                issues += 1;
            } else {
                println!("\x1b[32m[OK] Permissões do diretório de configuração corretas.\x1b[0m");
            }
        }
    }

    // 2. Load configurations
    let servers = load_servers();
    let keys = load_keys();
    println!("Total de servidores cadastrados: {}", servers.len());
    println!("Total de chaves cadastradas:     {}", keys.len());

    // 3. Verify key files
    for key in &keys {
        let path = PathBuf::from(&key.path);
        if !path.exists() {
            println!("\x1b[31m[ERRO] Arquivo de chave privada não encontrado: {} (Chave: {})\x1b[0m", key.path, key.name);
            issues += 1;
        } else {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = fs::metadata(&path) {
                    let mode = metadata.permissions().mode() & 0o777;
                    if mode != 0o600 {
                        println!("\x1b[33m[AVISO] Chave privada '{}' tem permissões muito abertas: {:o} (deveria ser 600)\x1b[0m", key.name, mode);
                        issues += 1;
                    }
                }
            }
        }
        let pub_path = path.with_extension("pub");
        if !pub_path.exists() {
            println!("\x1b[31m[ERRO] Arquivo de chave pública não encontrado: {}\x1b[0m", pub_path.display());
            issues += 1;
        }
    }

    // 4. Verify server links
    for server in &servers {
        if let Some(ref kn) = server.key_name {
            if !keys.iter().any(|k| &k.name == kn) {
                println!("\x1b[31m[ERRO] Servidor '{}' aponta para chave inexistente '{}'\x1b[0m", server.name, kn);
                issues += 1;
            }
        }

        // Test TCP connectivity briefly
        print!("Testando conexão TCP com '{}' ({})... ", server.name, server.host);
        let _ = io::stdout().flush();
        let addr = format!("{}:{}", server.host, server.port);
        let online = if let Ok(addrs) = addr.to_socket_addrs() {
            let mut success = false;
            for a in addrs {
                if TcpStream::connect_timeout(&a, Duration::from_secs(2)).is_ok() {
                    success = true;
                    break;
                }
            }
            success
        } else {
            false
        };

        if online {
            println!("\x1b[32mPorta aberta.\x1b[0m");
        } else {
            println!("\x1b[33mInacessível (Offline/Timeout).\x1b[0m");
            issues += 1;
        }
    }

    println!("-------------------------------------------");
    if issues == 0 {
        println!("\x1b[32mNenhum problema encontrado. O SSHX está operando perfeitamente!\x1b[0m");
    } else {
        println!("\x1b[33mEncontrados {} avisos/erros. Por favor, verifique os alertas acima.\x1b[0m", issues);
    }

    Ok(())
}
