mod config;
mod commands;

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "sshx")]
#[command(about = "SSHX - Gerenciador moderno e simples de conexões SSH", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Adiciona um novo servidor ao SSHX
    Add {
        /// Nome amigável do servidor
        nome: String,
    },
    /// Conecta-se a um servidor
    Connect {
        /// Nome do servidor
        nome: String,
    },
    /// Lista todos os servidores cadastrados
    List,
    /// Alias para 'list'
    Ls,
    /// Exibe informações detalhadas sobre um servidor
    Info {
        /// Nome do servidor
        nome: String,
    },
    /// Remove um servidor da configuração local
    Remove {
        /// Nome do servidor
        nome: String,
    },
    /// Renomeia um servidor
    Rename {
        /// Nome atual do servidor
        atual: String,
        /// Novo nome do servidor
        novo: String,
    },
    /// Copia arquivos de/para o servidor (ex: copy arquivo.zip producao:/opt/app)
    Copy {
        /// Origem do arquivo
        origem: String,
        /// Destino do arquivo
        destino: String,
    },
    /// Executa um comando remoto
    Exec {
        /// Nome do servidor
        servidor: String,
        /// Comando a ser executado
        comando: String,
    },
    /// Abre um terminal remoto
    Shell {
        /// Nome do servidor
        servidor: String,
    },
    /// Instala uma chave pública no servidor
    #[command(name = "install-key")]
    InstallKey {
        /// Nome do servidor
        servidor: String,
    },
    /// Lista todas as chaves cadastradas
    Keys,
    /// Gerenciamento de chaves SSH
    Key {
        #[command(subcommand)]
        command: KeyCommands,
    },
    /// Sincroniza chaves públicas com todos os servidores
    Sync,
    /// Analisa problemas na configuração
    Doctor,
    /// Verifica se um servidor está acessível
    Ping {
        /// Nome do servidor
        servidor: String,
    },
    /// Exibe a configuração SSH equivalente para o servidor
    Config {
        /// Nome do servidor
        servidor: String,
    },
    /// Exporta toda a configuração do SSHX
    Export {
        /// Formato de exportação: yaml, json, toml
        #[arg(short, long, default_value = "yaml")]
        format: String,
    },
    /// Importa configurações de um arquivo
    Import {
        /// Caminho do arquivo de configuração
        arquivo: String,
    },
    /// Gerenciamento do SSH Agent
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
}

#[derive(Subcommand)]
enum KeyCommands {
    /// Cria uma nova chave SSH
    Create {
        /// Nome da chave
        nome: String,
    },
    /// Remove uma chave localmente
    Delete {
        /// Nome da chave
        nome: String,
    },
    /// Rotaciona a chave de um servidor
    Rotate {
        /// Nome do servidor
        servidor: String,
    },
}

#[derive(Subcommand)]
enum AgentCommands {
    /// Inicia o SSH Agent
    Start,
    /// Finaliza o SSH Agent
    Stop,
    /// Adiciona uma chave ao Agent
    Add {
        /// Nome da chave
        chave: String,
    },
    /// Remove uma chave do Agent
    Remove {
        /// Nome da chave
        chave: String,
    },
    /// Lista as chaves carregadas no Agent
    List,
}

fn main() {
    // Enable directory initialization
    if let Err(e) = config::init_dirs() {
        eprintln!("Erro ao inicializar diretórios do SSHX: {}", e);
        process::exit(1);
    }

    // Pre-processing to support raw server names: `sshx producao` -> `sshx connect producao`
    let mut args: Vec<String> = std::env::args().collect();
    let known_subcommands = [
        "add", "connect", "list", "ls", "info", "remove", "rename", "copy",
        "exec", "shell", "install-key", "keys", "key", "sync", "doctor",
        "ping", "config", "export", "import", "agent"
    ];

    if args.len() > 1 {
        let first_arg = &args[1];
        // If the first argument is not a known subcommand and doesn't start with a dash
        if !known_subcommands.contains(&first_arg.as_str()) && !first_arg.starts_with('-') {
            args.insert(1, "connect".to_string());
        }
    }

    let cli = Cli::parse_from(args);

    let result = match cli.command {
        Commands::Add { nome } => commands::server::add_server(&nome),
        Commands::Connect { nome } => commands::server::connect_server(&nome),
        Commands::List => commands::server::list_servers(),
        Commands::Ls => commands::server::list_servers(),
        Commands::Info { nome } => commands::server::info_server(&nome),
        Commands::Remove { nome } => commands::server::remove_server(&nome),
        Commands::Rename { atual, novo } => commands::server::rename_server(&atual, &novo),
        Commands::Copy { origem, destino } => commands::utils::copy_files(&origem, &destino),
        Commands::Exec { servidor, comando } => commands::utils::exec_command(&servidor, &comando),
        Commands::Shell { servidor } => commands::server::connect_server(&servidor),
        Commands::InstallKey { servidor } => commands::key::install_key_on_server(&servidor),
        Commands::Keys => commands::key::list_keys(),
        Commands::Key { command } => match command {
            KeyCommands::Create { nome } => commands::key::create_key(&nome).map(|_| ()),
            KeyCommands::Delete { nome } => commands::key::delete_key(&nome),
            KeyCommands::Rotate { servidor } => commands::key::rotate_key(&servidor),
        },
        Commands::Sync => commands::key::sync_keys(),
        Commands::Doctor => commands::utils::run_doctor(),
        Commands::Ping { servidor } => commands::utils::ping_server(&servidor),
        Commands::Config { servidor } => commands::utils::show_config(&servidor),
        Commands::Export { format } => commands::utils::export_config(&format),
        Commands::Import { arquivo } => commands::utils::import_config(&arquivo),
        Commands::Agent { command } => match command {
            AgentCommands::Start => commands::agent::start(),
            AgentCommands::Stop => commands::agent::stop(),
            AgentCommands::Add { chave } => commands::agent::add(&chave),
            AgentCommands::Remove { chave } => commands::agent::remove(&chave),
            AgentCommands::List => commands::agent::list(),
        },
    };

    if let Err(e) = result {
        eprintln!("\x1b[31mErro: {}\x1b[0m", e);
        process::exit(1);
    }
}
