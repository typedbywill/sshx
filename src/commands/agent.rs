use crate::config::{load_agent_env, load_keys, remove_agent_env, save_agent_env, AgentEnv};
use std::io;
use std::process::Command;

#[cfg(unix)]
fn is_pid_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}

#[cfg(not(unix))]
fn is_pid_alive(pid: u32) -> bool {
    if let Ok(output) = Command::new("tasklist")
        .args(&["/FI", &format!("PID eq {}", pid)])
        .output()
    {
        let out_str = String::from_utf8_lossy(&output.stdout);
        out_str.contains(&pid.to_string())
    } else {
        false
    }
}

pub fn setup_agent_env(cmd: &mut Command) {
    let _ = ensure_agent_and_key(cmd, None);
}

pub fn start_silent() -> io::Result<AgentEnv> {
    let output = Command::new("ssh-agent").arg("-s").output()?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to start ssh-agent: {}", err_msg),
        ));
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
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to parse ssh-agent environment variables.",
        ));
    }

    let env = AgentEnv {
        socket: socket.clone(),
        pid,
    };
    save_agent_env(&env)?;

    Ok(env)
}

pub fn start() -> io::Result<()> {
    let env = start_silent()?;

    // Output bash-compatible export statements for eval
    println!("SSH_AUTH_SOCK={}; export SSH_AUTH_SOCK;", env.socket);
    println!("SSH_AGENT_PID={}; export SSH_AGENT_PID;", env.pid);
    println!("echo Agent pid {};", env.pid);

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
        let status = Command::new("kill").arg(env.pid.to_string()).status();

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

pub fn ensure_agent_and_key(cmd: &mut Command, key_path: Option<&str>) -> io::Result<()> {
    let mut active_socket = None;
    let mut active_pid = None;

    // 1. Check if system agent is in environment
    if let Ok(sock) = std::env::var("SSH_AUTH_SOCK") {
        if !sock.is_empty() {
            active_socket = Some(sock);
            if let Ok(p) = std::env::var("SSH_AGENT_PID") {
                active_pid = p.parse::<u32>().ok();
            }
        }
    }

    // 2. If not, check agent.env
    if active_socket.is_none() {
        if let Some(env) = load_agent_env() {
            if is_pid_alive(env.pid) {
                active_socket = Some(env.socket);
                active_pid = Some(env.pid);
            } else {
                let _ = remove_agent_env();
            }
        }
    }

    // 3. Start a new managed agent if none exists
    let (sock, pid) = match (active_socket, active_pid) {
        (Some(s), Some(p)) => (s, p),
        (Some(s), None) => (s, 0),
        _ => {
            let env = start_silent()?;
            (env.socket, env.pid)
        }
    };

    // 4. Set environment on target command
    cmd.env("SSH_AUTH_SOCK", &sock);
    if pid > 0 {
        cmd.env("SSH_AGENT_PID", pid.to_string());
    }

    // 5. Load key if not already present
    if let Some(kp) = key_path {
        let mut check_cmd = Command::new("ssh-add");
        check_cmd.env("SSH_AUTH_SOCK", &sock);
        if pid > 0 {
            check_cmd.env("SSH_AGENT_PID", pid.to_string());
        }
        check_cmd.arg("-l");

        let check_output = check_cmd.output()?;
        let is_loaded = if check_output.status.success() {
            let output_str = String::from_utf8_lossy(&check_output.stdout);
            let target_fp = crate::commands::key::get_fingerprint(kp).unwrap_or_default();
            output_str.contains(kp) || (!target_fp.is_empty() && output_str.contains(&target_fp))
        } else {
            false
        };

        if !is_loaded {
            let mut add_cmd = Command::new("ssh-add");
            add_cmd.env("SSH_AUTH_SOCK", &sock);
            if pid > 0 {
                add_cmd.env("SSH_AGENT_PID", pid.to_string());
            }
            add_cmd.arg(kp);

            let add_status = add_cmd.status()?;
            if !add_status.success() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Falha ao carregar chave no SSH Agent.",
                ));
            }
        }
    }

    Ok(())
}

pub fn add(key_name: &str) -> io::Result<()> {
    let keys = load_keys();
    let key = keys.iter().find(|k| k.name == key_name).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Chave '{}' não encontrada no keys.yaml", key_name),
        )
    })?;

    let mut cmd = Command::new("ssh-add");
    setup_agent_env(&mut cmd);
    cmd.arg(&key.path);

    let status = cmd.status()?;
    if status.success() {
        println!("Chave '{}' adicionada ao SSH Agent.", key_name);
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Falha ao adicionar a chave ao SSH Agent.",
        ))
    }
}

pub fn remove(key_name: &str) -> io::Result<()> {
    let keys = load_keys();
    let key = keys.iter().find(|k| k.name == key_name).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("Chave '{}' não encontrada no keys.yaml", key_name),
        )
    })?;

    let mut cmd = Command::new("ssh-add");
    setup_agent_env(&mut cmd);
    cmd.arg("-d").arg(&key.path);

    let status = cmd.status()?;
    if status.success() {
        println!("Chave '{}' removida do SSH Agent.", key_name);
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Falha ao remover a chave do SSH Agent.",
        ))
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
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Falha ao listar chaves no SSH Agent: {}", stderr.trim()),
            ))
        }
    }
}
