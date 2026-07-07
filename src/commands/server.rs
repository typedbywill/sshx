use crate::commands::agent::ensure_agent_and_key;
use crate::commands::key::{create_key, get_fingerprint, install_key_on_server};
use crate::config::{add_history, load_keys, load_servers, resolve_key, save_servers, Server};
use chrono::Local;
use dialoguer::{Confirm, Input, Select};
use std::io::{self, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::process::Command;
use std::time::{Duration, Instant};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ServerRow {
    #[tabled(rename = "Nome")]
    name: String,
    #[tabled(rename = "Host")]
    host: String,
    #[tabled(rename = "Usuário")]
    user: String,
    #[tabled(rename = "Porta")]
    port: u16,
    #[tabled(rename = "Chave")]
    key: String,
    #[tabled(rename = "Ambiente")]
    environment: String,
    #[tabled(rename = "Última Conexão")]
    last_connected: String,
}

pub fn add_server(name: &str) -> io::Result<()> {
    let mut servers = load_servers();
    if servers.iter().any(|s| s.name == name) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Servidor com nome '{}' já está cadastrado.", name),
        ));
    }

    println!("Adicionando novo servidor: {}", name);

    let host = Input::<String>::new()
        .with_prompt("Host (IP ou domínio)")
        .interact_text()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let user = Input::<String>::new()
        .with_prompt("Usuário")
        .default("root".to_string())
        .interact_text()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let port = Input::<u16>::new()
        .with_prompt("Porta")
        .default(22)
        .interact_text()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let env_input = Input::<String>::new()
        .with_prompt("Ambiente (ex: Produção, Homelab) [opcional]")
        .allow_empty(true)
        .interact_text()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let environment = if env_input.trim().is_empty() {
        None
    } else {
        Some(env_input.trim().to_string())
    };

    // Authentication method selection
    let auth_options = vec![
        "Criar uma nova chave SSH Ed25519",
        "Usar uma chave SSH cadastrada existente",
        "Usar chave padrão do sistema / Sem chave específica",
    ];

    let selection = Select::new()
        .with_prompt("Escolha o método de autenticação")
        .items(&auth_options)
        .default(0)
        .interact()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut key_name = None;

    match selection {
        0 => {
            // Create a new key
            let default_key_name = format!("{}", name);
            let k_name = Input::<String>::new()
                .with_prompt("Nome para a nova chave")
                .default(default_key_name)
                .interact_text()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

            match create_key(&k_name) {
                Ok(_) => {
                    key_name = Some(k_name);
                }
                Err(e) => {
                    println!(
                        "Erro ao criar chave, continuando sem chave específica: {}",
                        e
                    );
                }
            }
        }
        1 => {
            // Use existing key
            let keys = load_keys();
            if keys.is_empty() {
                println!("Nenhuma chave cadastrada encontrada. Criando uma nova chave...");
                let default_key_name = format!("{}", name);
                let k_name = Input::<String>::new()
                    .with_prompt("Nome para a nova chave")
                    .default(default_key_name)
                    .interact_text()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                match create_key(&k_name) {
                    Ok(_) => {
                        key_name = Some(k_name);
                    }
                    Err(e) => {
                        println!("Erro ao criar chave: {}", e);
                    }
                }
            } else {
                let key_names: Vec<String> = keys.iter().map(|k| k.name.clone()).collect();
                let key_select = Select::new()
                    .with_prompt("Selecione a chave")
                    .items(&key_names)
                    .default(0)
                    .interact()
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                key_name = Some(key_names[key_select].clone());
            }
        }
        _ => {
            // No key / default key
        }
    }

    let new_server = Server {
        name: name.to_string(),
        host,
        user,
        port,
        key_name: key_name.clone(),
        environment,
        created_at: Local::now().to_rfc3339(),
        last_connected: None,
    };

    servers.push(new_server);
    save_servers(&servers)?;
    println!("Servidor '{}' salvo com sucesso localmente.", name);

    // Ask to install key on remote server
    if key_name.is_some() {
        let install_confirm = Confirm::new()
            .with_prompt("Deseja instalar a chave pública no servidor remoto agora?")
            .default(true)
            .interact()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        if install_confirm {
            if let Err(e) = install_key_on_server(name) {
                println!("Aviso: Falha ao instalar a chave no servidor: {}", e);
            }
        }
    }

    Ok(())
}

pub fn list_servers() -> io::Result<()> {
    let servers = load_servers();
    let mut rows = Vec::new();

    for s in servers {
        rows.push(ServerRow {
            name: s.name,
            host: s.host,
            user: s.user,
            port: s.port,
            key: s.key_name.unwrap_or_else(|| "Padrão".to_string()),
            environment: s.environment.unwrap_or_else(|| "-".to_string()),
            last_connected: s.last_connected.unwrap_or_else(|| "Nunca".to_string()),
        });
    }

    if rows.is_empty() {
        println!("Nenhum servidor cadastrado.");
    } else {
        println!("{}", Table::new(rows).to_string());
    }

    Ok(())
}

pub fn connect_server(name: &str) -> io::Result<()> {
    let mut servers = load_servers();
    let index = servers.iter().position(|s| s.name == name).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Servidor '{}' não encontrado.", name),
        )
    })?;

    let server = &mut servers[index];
    server.last_connected = Some(Local::now().to_rfc3339());
    let server_copy = server.clone();

    // Save servers to update last_connected
    save_servers(&servers)?;
    // Record history
    let _ = add_history(name);

    println!(
        "Conectando-se ao servidor '{}' ({})...",
        server_copy.name, server_copy.host
    );

    let resolved_key = resolve_key(server_copy.key_name.as_deref());
    let key_path = resolved_key.as_ref().map(|k| k.path.as_str());

    let mut cmd = Command::new("ssh");
    ensure_agent_and_key(&mut cmd, key_path)?;
    crate::commands::utils::add_multiplexing_opts(&mut cmd, name);

    if let Some(kp) = key_path {
        cmd.arg("-i").arg(kp);
        cmd.arg("-o").arg("IdentitiesOnly=yes");
    }

    cmd.arg("-p").arg(server_copy.port.to_string());
    cmd.arg(format!("{}@{}", server_copy.user, server_copy.host));

    let start_time = Instant::now();

    #[cfg(unix)]
    let status = {
        let sigint_old = unsafe { libc::signal(libc::SIGINT, libc::SIG_IGN) };
        let sigquit_old = unsafe { libc::signal(libc::SIGQUIT, libc::SIG_IGN) };

        let mut child = cmd.spawn()?;
        let res = child.wait();

        unsafe {
            libc::signal(libc::SIGINT, sigint_old);
            libc::signal(libc::SIGQUIT, sigquit_old);
        }
        res?
    };

    #[cfg(not(unix))]
    let status = {
        let mut child = cmd.spawn()?;
        child.wait()?
    };

    let duration = start_time.elapsed();
    print_connection_summary(&server_copy, duration, status);

    if let Some(code) = status.code() {
        if code != 0 {
            std::process::exit(code);
        }
    } else {
        std::process::exit(130);
    }

    Ok(())
}

fn print_connection_summary(server: &Server, duration: Duration, status: std::process::ExitStatus) {
    let duration_secs = duration.as_secs();
    let hours = duration_secs / 3600;
    let minutes = (duration_secs % 3600) / 60;
    let seconds = duration_secs % 60;

    let duration_str = if hours > 0 {
        format!("{:02}h {:02}m {:02}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{:02}m {:02}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    };

    let auth_str = if let Some(ref key_name) = server.key_name {
        format!("Chave SSH '{}'", key_name)
    } else {
        "Padrão do sistema (Chave/Senha/Agente)".to_string()
    };

    let server_info = format!("{} ({})", server.name, server.host);
    let server_info = if server_info.chars().count() > 38 {
        let mut truncated: String = server_info.chars().take(35).collect();
        truncated.push_str("...");
        truncated
    } else {
        server_info
    };

    let auth_str = if auth_str.chars().count() > 38 {
        let mut truncated: String = auth_str.chars().take(35).collect();
        truncated.push_str("...");
        truncated
    } else {
        auth_str
    };

    let code_str;
    let (status_color, status_text) = match status.code() {
        Some(0) => ("\x1b[32m", "Sucesso (Código 0)"),
        Some(code) => {
            code_str = format!("Erro (Código {})", code);
            ("\x1b[31m", code_str.as_str())
        }
        None => ("\x1b[33m", "Terminado por sinal"),
    };

    println!("\n\x1b[36m┌────────────────────────────────────────────────────────┐\x1b[0m");
    println!("\x1b[36m│\x1b[0m   \x1b[1;35mConexão SSH encerrada com o servidor\x1b[0m                  \x1b[36m│\x1b[0m");
    println!("\x1b[36m├────────────────────────────────────────────────────────┤\x1b[0m");
    println!("\x1b[36m│\x1b[0m  \x1b[1mServidor:\x1b[0m      {:<38} \x1b[36m│\x1b[0m", server_info);
    println!("\x1b[36m│\x1b[0m  \x1b[1mDuração:\x1b[0m       {:<38} \x1b[36m│\x1b[0m", duration_str);
    println!("\x1b[36m│\x1b[0m  \x1b[1mAutenticação:\x1b[0m  {:<38} \x1b[36m│\x1b[0m", auth_str);
    println!("\x1b[36m│\x1b[0m  \x1b[1mStatus:\x1b[0m        {}{:<38}\x1b[0m \x1b[36m│\x1b[0m", status_color, status_text);
    println!("\x1b[36m└────────────────────────────────────────────────────────┘\x1b[0m\n");
}

pub fn info_server(name: &str) -> io::Result<()> {
    let servers = load_servers();
    let server = servers.iter().find(|s| s.name == name).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Servidor '{}' não encontrado.", name),
        )
    })?;

    println!("--- Informações do Servidor: {} ---", server.name);
    println!("Host:            {}", server.host);
    println!("Usuário:         {}", server.user);
    println!("Porta:           {}", server.port);
    println!(
        "Ambiente:        {}",
        server.environment.as_deref().unwrap_or("-")
    );
    println!("Criado em:       {}", server.created_at);
    println!(
        "Última conexão:  {}",
        server.last_connected.as_deref().unwrap_or("Nunca")
    );

    // Key information
    if let Some(ref k_name) = server.key_name {
        println!("Chave Associada: {}", k_name);
        let keys = load_keys();
        if let Some(k) = keys.iter().find(|k| &k.name == k_name) {
            println!("Caminho Chave:   {}", k.path);
            if let Ok(fp) = get_fingerprint(&k.path) {
                println!("Fingerprint:     {}", fp);
            }
        }
    } else {
        println!("Chave Associada: Padrão do sistema");
    }

    // Ping check
    print!("Status Conexão:  Verificando...");
    let _ = io::stdout().flush();

    let addr = format!("{}:{}", server.host, server.port);
    let timeout = Duration::from_secs(3);

    // Resolve host address and try TCP connection
    let online = if let Ok(addrs) = addr.to_socket_addrs() {
        let mut success = false;
        for a in addrs {
            if TcpStream::connect_timeout(&a, timeout).is_ok() {
                success = true;
                break;
            }
        }
        success
    } else {
        false
    };

    if online {
        println!("\x1b[32mOnline (Porta aberta)\x1b[0m");
    } else {
        println!("\x1b[31mInacessível / Offline\x1b[0m");
    }

    Ok(())
}

pub fn remove_server(name: &str) -> io::Result<()> {
    let mut servers = load_servers();
    let index = servers.iter().position(|s| s.name == name).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Servidor '{}' não encontrado.", name),
        )
    })?;

    servers.remove(index);
    save_servers(&servers)?;
    println!("Servidor '{}' removido com sucesso.", name);
    Ok(())
}

pub fn rename_server(old_name: &str, new_name: &str) -> io::Result<()> {
    let mut servers = load_servers();

    // Check if new name already exists
    if servers.iter().any(|s| s.name == new_name) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Um servidor com o nome '{}' já existe.", new_name),
        ));
    }

    let index = servers
        .iter()
        .position(|s| s.name == old_name)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Servidor '{}' não encontrado.", old_name),
            )
        })?;

    servers[index].name = new_name.to_string();
    save_servers(&servers)?;
    println!(
        "Servidor renomeado de '{}' para '{}' com sucesso.",
        old_name, new_name
    );
    Ok(())
}
