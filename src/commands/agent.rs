use std::process::Command;
use std::io;
use crate::config::{save_agent_env, load_agent_env, remove_agent_env, AgentEnv, load_keys};

pub fn setup_agent_env(cmd: &mut Command) {
    // If the shell already has SSH_AUTH_SOCK, use it.
    if std::env::var("SSH_AUTH_SOCK").is_ok() {
        return;
    }
    // Otherwise, check if we have a saved agent.env.
    if let Some(env) = load_agent_env() {
        cmd.env("SSH_AUTH_SOCK", &env.socket);
        cmd.env("SSH_AGENT_PID", env.pid.to_string());
    }
}

pub fn start() -> io::Result<()> {
    let output = Command::new("ssh-agent")
        .arg("-s")
        .output()?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to start ssh-agent: {}", err_msg)));
    }

    let out_str = String::from_utf8_lossy(&output.stdout);
    let mut socket = String::new();
    let mut pid: u32 = 0;

    for line in out_str.lines() {
        if line.starts_with("SSH_AUTH_SOCK=") {
            if let Some(val) = line.split(';').next() {
                socket = val.trim_start_matches("SSH_AUTH_SOCK=").to_string();
            }
        } else if line.starts_with("SSH_AGENT_PID=") {
            if let Some(val) = line.split(';').next() {
                if let Ok(p) = val.trim_start_matches("SSH_AGENT_PID=").parse::<u32>() {
                    pid = p;
                }
            }
        }
    }

    if socket.is_empty() || pid == 0 {
        return Err(io::Error::new(io::ErrorKind::Other, "Failed to parse ssh-agent environment variables."));
    }

    let env = AgentEnv { socket: socket.clone(), pid };
    save_agent_env(&env)?;

    // Output bash-compatible export statements for eval
    println!("SSH_AUTH_SOCK={}; export SSH_AUTH_SOCK;", socket);
    println!("SSH_AGENT_PID={}; export SSH_AGENT_PID;", pid);
    println!("echo Agent pid {};", pid);

    Ok(())
}

pub fn stop() -> io::Result<()> {
    let env = match load_agent_env() {
        Some(e) => e,
        None => {
            println!("Nenhum SSH Agent registrado pelo SSHX ativo.");
            return Ok(());
        }
    };

    #[cfg(unix)]
    {
        // Try to kill using std::process::Command
        let status = Command::new("kill")
            .arg(env.pid.to_string())
            .status();
        
        match status {
            Ok(s) if s.success() => {
                println!("SSH Agent (PID {}) finalizado com sucesso.", env.pid);
            }
            _ => {
                // Fallback to sending SIGTERM via libc if kill command fails
                unsafe {
                    libc::kill(env.pid as libc::pid_t, libc::SIGTERM);
                }
                println!("SSH Agent (PID {}) finalizado.", env.pid);
            }
        }
    }

    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(&["/F", "/PID", &env.pid.to_string()])
            .status();
        println!("SSH Agent (PID {}) finalizado.", env.pid);
    }

    remove_agent_env()?;
    Ok(())
}

pub fn add(key_name: &str) -> io::Result<()> {
    // Find the key path in keys.yaml
    let keys = load_keys();
    let key = keys.iter().find(|k| k.name == key_name)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Chave '{}' não encontrada no keys.yaml", key_name)))?;

    let mut cmd = Command::new("ssh-add");
    setup_agent_env(&mut cmd);
    cmd.arg(&key.path);

    let status = cmd.status()?;
    if status.success() {
        println!("Chave '{}' adicionada ao SSH Agent.", key_name);
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Falha ao adicionar a chave ao SSH Agent."))
    }
}

pub fn remove(key_name: &str) -> io::Result<()> {
    let keys = load_keys();
    let key = keys.iter().find(|k| k.name == key_name)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Chave '{}' não encontrada no keys.yaml", key_name)))?;

    let mut cmd = Command::new("ssh-add");
    setup_agent_env(&mut cmd);
    cmd.arg("-d").arg(&key.path);

    let status = cmd.status()?;
    if status.success() {
        println!("Chave '{}' removida do SSH Agent.", key_name);
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Falha ao remover a chave do SSH Agent."))
    }
}

pub fn list() -> io::Result<()> {
    let mut cmd = Command::new("ssh-add");
    setup_agent_env(&mut cmd);
    cmd.arg("-l");

    let output = cmd.output()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("The agent has no identities") || output.stdout.is_empty() {
            println!("Nenhuma chave carregada no SSH Agent.");
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, format!("Falha ao listar chaves no SSH Agent: {}", stderr.trim())))
        }
    }
}
