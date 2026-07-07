use std::process::Command;
use std::io;
use std::fs;
use std::path::PathBuf;
use crate::config::{
    get_keys_dir, load_keys, save_keys, KeyInfo, load_servers, save_servers
};
use crate::commands::agent::{ensure_agent_and_key, setup_agent_env};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct KeyRow {
    #[tabled(rename = "Nome")]
    name: String,
    #[tabled(rename = "Tipo")]
    key_type: String,
    #[tabled(rename = "Fingerprint")]
    fingerprint: String,
    #[tabled(rename = "Caminho")]
    path: String,
    #[tabled(rename = "Servidores")]
    servers_count: usize,
}

pub fn list_keys() -> io::Result<()> {
    let keys = load_keys();
    let servers = load_servers();

    let mut rows = Vec::new();
    for key in keys {
        // Count how many servers use this key
        let count = servers.iter()
            .filter(|s| s.key_name.as_deref() == Some(&key.name))
            .count();

        // Get fingerprint
        let fingerprint = get_fingerprint(&key.path)
            .unwrap_or_else(|_| "Não disponível".to_string());

        rows.push(KeyRow {
            name: key.name.clone(),
            key_type: key.key_type.clone(),
            fingerprint,
            path: key.path.clone(),
            servers_count: count,
        });
    }

    if rows.is_empty() {
        println!("Nenhuma chave cadastrada.");
    } else {
        println!("{}", Table::new(rows).to_string());
    }

    Ok(())
}

pub fn create_key(name: &str) -> io::Result<PathBuf> {
    let keys_dir = get_keys_dir();
    let key_path = keys_dir.join(name);

    if key_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Chave com nome '{}' já existe.", name)
        ));
    }

    println!("Gerando chave Ed25519 para '{}'...", name);
    let output = Command::new("ssh-keygen")
        .args(&[
            "-t", "ed25519",
            "-N", "", // empty passphrase
            "-f", key_path.to_str().unwrap()
        ])
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Falha ao gerar chave SSH: {}", err.trim())
        ));
    }

    // Set correct permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
        let pub_key = key_path.with_extension("pub");
        let _ = fs::set_permissions(&pub_key, fs::Permissions::from_mode(0o644));
    }

    // Register in keys.yaml
    let mut keys = load_keys();
    keys.push(KeyInfo {
        name: name.to_string(),
        key_type: "Ed25519".to_string(),
        path: key_path.to_str().unwrap().to_string(),
    });
    save_keys(&keys)?;

    println!("Chave '{}' criada com sucesso em: {}", name, key_path.display());
    Ok(key_path)
}

pub fn delete_key(name: &str) -> io::Result<()> {
    let mut keys = load_keys();
    let index = keys.iter().position(|k| k.name == name)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Chave '{}' não encontrada no keys.yaml", name)
        ))?;

    let key = keys.remove(index);

    // Delete files
    let priv_path = PathBuf::from(&key.path);
    let pub_path = priv_path.with_extension("pub");

    if priv_path.exists() {
        fs::remove_file(priv_path)?;
    }
    if pub_path.exists() {
        fs::remove_file(pub_path)?;
    }

    save_keys(&keys)?;
    println!("Chave '{}' removida com sucesso.", name);
    Ok(())
}

pub fn get_fingerprint(private_key_path: &str) -> io::Result<String> {
    let pub_path = format!("{}.pub", private_key_path);
    let path_to_use = if PathBuf::from(&pub_path).exists() {
        pub_path
    } else {
        private_key_path.to_string()
    };

    let output = Command::new("ssh-keygen")
        .args(&["-lf", &path_to_use])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Output looks like: "256 SHA256:xxxxxx usuario@host (ED25519)"
        let parts: Vec<&str> = stdout.split_whitespace().collect();
        if parts.len() >= 2 {
            return Ok(parts[1].to_string());
        }
        Ok(stdout.trim().to_string())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Failed to get fingerprint"))
    }
}

pub fn install_key_on_server(server_name: &str) -> io::Result<()> {
    let servers = load_servers();
    let server = servers.iter().find(|s| s.name == server_name)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Servidor '{}' não encontrado.", server_name)
        ))?;

    let key_name = server.key_name.as_ref()
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Servidor '{}' não possui chave SSH associada.", server_name)
        ))?;

    let keys = load_keys();
    let key = keys.iter().find(|k| &k.name == key_name)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Chave '{}' associada ao servidor não foi encontrada no keys.yaml", key_name)
        ))?;

    println!("Instalando chave pública '{}' no servidor '{}' ({})...", key_name, server.name, server.host);

    let pub_key_path = format!("{}.pub", key.path);
    if !PathBuf::from(&pub_key_path).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Chave pública não encontrada em: {}", pub_key_path)
        ));
    }

    // Try ssh-copy-id first
    let mut cmd = Command::new("ssh-copy-id");
    setup_agent_env(&mut cmd);
    cmd.args(&[
        "-i", &pub_key_path,
        "-p", &server.port.to_string(),
        &format!("{}@{}", server.user, server.host)
    ]);

    let status = cmd.status();
    match status {
        Ok(s) if s.success() => {
            println!("Chave pública instalada com sucesso via ssh-copy-id.");
            Ok(())
        }
        _ => {
            println!("ssh-copy-id falhou ou não está instalado. Tentando método manual de append remoto...");
            // Manual method: read pub key and write it remotely
            let pub_key_content = fs::read_to_string(&pub_key_path)?;
            let pub_key_content = pub_key_content.trim();

            let remote_command = format!(
                "mkdir -p ~/.ssh && chmod 700 ~/.ssh && echo '{}' >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys",
                pub_key_content
            );

            let mut ssh_cmd = Command::new("ssh");
            setup_agent_env(&mut ssh_cmd);
            ssh_cmd.args(&[
                "-p", &server.port.to_string(),
                &format!("{}@{}", server.user, server.host),
                &remote_command
            ]);

            let manual_status = ssh_cmd.status()?;
            if manual_status.success() {
                println!("Chave pública instalada com sucesso manualmente.");
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Falha ao instalar chave pública no servidor remoto."
                ))
            }
        }
    }
}

pub fn rotate_key(server_name: &str) -> io::Result<()> {
    let mut servers = load_servers();
    let server_index = servers.iter().position(|s| s.name == server_name)
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::NotFound,
            format!("Servidor '{}' não encontrado.", server_name)
        ))?;
    
    let server = &servers[server_index];
    let old_key_name = server.key_name.clone();

    // Generate a new key name
    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    let new_key_name = format!("{}_rot_{}", server_name, timestamp);

    println!("Iniciando rotação de chave para o servidor '{}'...", server_name);

    // Create the new key
    let new_key_path = create_key(&new_key_name)?;
    let new_pub_path = new_key_path.with_extension("pub");
    let new_pub_content = fs::read_to_string(&new_pub_path)?;
    let new_pub_content = new_pub_content.trim();

    // 1. Install the new key using the old key for auth
    println!("Instalando nova chave no servidor remoto...");
    let mut old_key_path = None;
    if let Some(ref old_name) = old_key_name {
        let keys = load_keys();
        if let Some(old_k) = keys.iter().find(|k| &k.name == old_name) {
            old_key_path = Some(old_k.path.clone());
        }
    }

    let mut append_cmd = Command::new("ssh");
    ensure_agent_and_key(&mut append_cmd, old_key_path.as_deref())?;
    
    if let Some(ref path) = old_key_path {
        append_cmd.args(&["-i", path]);
    }

    let remote_append_cmd = format!(
        "mkdir -p ~/.ssh && chmod 700 ~/.ssh && echo '{}' >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys",
        new_pub_content
    );
    append_cmd.args(&[
        "-p", &server.port.to_string(),
        &format!("{}@{}", server.user, server.host),
        &remote_append_cmd
    ]);

    let append_status = append_cmd.status()?;
    if !append_status.success() {
        // Cleanup generated key
        let _ = delete_key(&new_key_name);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Falha ao instalar a nova chave pública no servidor remoto."
        ));
    }

    // 2. Verify new key works
    println!("Verificando conexão com a nova chave...");
    let mut verify_cmd = Command::new("ssh");
    ensure_agent_and_key(&mut verify_cmd, Some(new_key_path.to_str().unwrap()))?;
    verify_cmd.args(&[
        "-i", new_key_path.to_str().unwrap(),
        "-o", "BatchMode=yes",
        "-o", "ConnectTimeout=5",
        "-p", &server.port.to_string(),
        &format!("{}@{}", server.user, server.host),
        "echo OK"
    ]);

    let verify_output = verify_cmd.output()?;
    if !verify_output.status.success() || String::from_utf8_lossy(&verify_output.stdout).trim() != "OK" {
        println!("AVISO: A verificação da nova chave falhou. A chave antiga permanece ativa.");
        // Try to revert by deleting new key from config (remotely it's appended but old still works)
        let _ = delete_key(&new_key_name);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "A nova chave não conseguiu se autenticar. Operação abortada."
        ));
    }

    // 3. Remove old key from remote authorized_keys if old key existed
    if let Some(ref old_name) = old_key_name {
        let keys = load_keys();
        if let Some(old_k) = keys.iter().find(|k| &k.name == old_name) {
            let old_pub_path = format!("{}.pub", old_k.path);
            if let Ok(old_pub_content) = fs::read_to_string(&old_pub_path) {
                // Extract just the key part (e.g. "ssh-ed25519 AAAA...")
                let parts: Vec<&str> = old_pub_content.split_whitespace().collect();
                if parts.len() >= 2 {
                    let old_key_base64 = parts[1];
                    println!("Removendo chave antiga do servidor remoto...");
                    
                    let remote_remove_cmd = format!(
                        "grep -v '{}' ~/.ssh/authorized_keys > ~/.ssh/authorized_keys.tmp && mv ~/.ssh/authorized_keys.tmp ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys",
                        old_key_base64
                    );

                    let mut remove_cmd = Command::new("ssh");
                    ensure_agent_and_key(&mut remove_cmd, Some(new_key_path.to_str().unwrap()))?;
                    remove_cmd.args(&[
                        "-i", new_key_path.to_str().unwrap(),
                        "-p", &server.port.to_string(),
                        &format!("{}@{}", server.user, server.host),
                        &remote_remove_cmd
                    ]);

                    let _ = remove_cmd.status();
                }
            }
        }
    }

    // 4. Update server config
    servers[server_index].key_name = Some(new_key_name.clone());
    save_servers(&servers)?;

    // 5. Clean up old key registry if requested or if no longer used
    if let Some(ref old_name) = old_key_name {
        let still_used = servers.iter().any(|s| s.key_name.as_ref() == Some(old_name));
        if !still_used {
            println!("Removendo chave antiga '{}' do inventário local...", old_name);
            let _ = delete_key(old_name);
        }
    }

    println!("Rotação concluída com sucesso! Servidor '{}' agora utiliza a chave '{}'.", server_name, new_key_name);
    Ok(())
}

pub fn sync_keys() -> io::Result<()> {
    let servers = load_servers();
    println!("Sincronizando chaves com todos os servidores cadastrados...");

    for server in &servers {
        if server.key_name.is_some() {
            println!("--- Servidor: {} ---", server.name);
            if let Err(e) = install_key_on_server(&server.name) {
                println!("Aviso: Falha ao sincronizar com '{}': {}", server.name, e);
            }
        }
    }

    println!("Sincronização concluída.");
    Ok(())
}
