# SSHX 🚀

> **SSHX** é um gerenciador moderno e intuitivo de conexões SSH em Rust que atua como uma camada de abstração sobre o OpenSSH. Ele simplifica tarefas repetitivas, automatiza configurações de chaves e oferece uma experiência limpa e consistente sem substituir a estabilidade e a segurança do OpenSSH tradicional.

---

## ✨ Características Principais

* **Interface Simples e Memorável**: Esqueça comandos longos com múltiplos parâmetros. Use nicknames curtos para seus servidores.
* **Sem Runtimes / Dependências**: Um binário único compilado em Rust que roda nativamente em Linux, macOS e Windows.
* **Totalmente Compatível**: Opera no topo do OpenSSH existente, utilizando seus próprios executáveis (`ssh`, `scp`, `ssh-agent`, `ssh-keygen`, `ssh-copy-id`).
* **Cadastro Interativo**: Assistente interativo para cadastro de servidores, com geração e instalação automática de chaves públicas.
* **Inventário Integrado de Chaves**: Gerencie chaves SSH modernas (Ed25519) e veja quais servidores utilizam cada chave.
* **Automação Inteligente de Agente**: SSH Agent persistido em arquivos de configuração locais. Mesmo que você abra novos terminais sem `eval $(ssh-agent -s)`, o SSHX gerencia e passa o socket do agente automaticamente.
* **Rotação e Sincronização**: Rotacione chaves em servidores com um único comando de forma segura, ou sincronize chaves públicas em lote.
* **Doctor Diagnóstico**: Auditoria automatizada de conectividade, chaves ausentes, e permissões do diretório de chaves.

---

## 📦 Estrutura de Diretórios

O SSHX organiza suas configurações de forma centralizada em:

`~/.config/sshx/` (Linux/macOS) ou `%USERPROFILE%\.config\sshx\` (Windows)

```text
~/.config/sshx/
├── servers.yaml       # Cadastro dos servidores
├── keys.yaml          # Metadados das chaves associadas
├── history.json       # Histórico de conexões efetuadas
├── agent.env          # Socket e PID do SSH Agent gerenciado
└── keys/              # Pasta segura contendo as chaves privadas/públicas
    ├── github
    ├── producao
    └── homelab
```

---

## 🛠️ Instalação e Requisitos

### Pré-requisitos
Certifique-se de ter as seguintes ferramentas de linha de comando instaladas (OpenSSH padrão):
* `ssh`
* `scp`
* `ssh-keygen`
* `ssh-copy-id` (opcional, mas recomendado)
* `ssh-agent` e `ssh-add`

### Compilando do Código Fonte
Clone este repositório e execute a compilação:

```bash
cargo build --release
```

O binário final estará localizado em `target/release/sshx`. Mova-o para seu `$PATH` (ex: `/usr/local/bin/`).

---

## 🚀 Guia Rápido de Uso

### 1. Cadastrar um Servidor
```bash
sshx add producao
```
O assistente irá solicitar:
* Host (IP ou domínio)
* Usuário (Ex: `ubuntu`, `root`)
* Porta (Padrão: `22`)
* Ambiente (Ex: `Produção`, `Homelab`)
* Método de Autenticação (Criar chave, usar existente, usar padrão do sistema)
* Instalação automática da chave pública no servidor remoto.

### 2. Conectar-se
```bash
sshx producao
```
ou
```bash
sshx connect producao
```

### 3. Listar Servidores Cadastrados
```bash
sshx list
# ou simplesmente
sshx ls
```

### 4. Diagnosticar Permissões e Conexões
```bash
sshx doctor
```

---

## 📚 Referência de Comandos

| Comando | Descrição |
| :--- | :--- |
| `sshx add <nome>` | Cadastra um novo servidor interativamente |
| `sshx connect <nome>` | Conecta-se a um servidor cadastrado |
| `sshx list` / `ls` | Lista todos os servidores |
| `sshx info <nome>` | Exibe informações detalhadas e testa conectividade |
| `sshx remove <nome>` | Remove um servidor localmente |
| `sshx rename <atual> <novo>` | Renomeia o atalho de um servidor |
| `sshx copy <origem> <destino>` | Copia arquivos via SCP (ex: `sshx copy arquivo.zip producao:/tmp`) |
| `sshx exec <servidor> "<cmd>"` | Executa um comando remotamente e exibe o resultado |
| `sshx ping <servidor>` | Testa conectividade e autenticação via SSH |
| `sshx config <servidor>` | Exibe o bloco equivalente do arquivo `~/.ssh/config` |
| `sshx export` | Exporta configurações para YAML, JSON ou TOML |
| `sshx import <arquivo>` | Importa servidores e chaves de arquivos exportados |

### Gerenciamento de Chaves (`sshx key`)
* `sshx keys`: Lista todas as chaves no inventário.
* `sshx key create <nome>`: Gera uma nova chave Ed25519 localmente.
* `sshx key delete <nome>`: Remove a chave do inventário local.
* `sshx key rotate <servidor>`: Rotaciona a chave de um servidor (gera nova, instala e desativa antiga de forma automatizada).
* `sshx install-key <servidor>`: Instala a chave pública associada no servidor remoto.
* `sshx sync`: Sincroniza em lote as chaves públicas configuradas.

### Gerenciamento do Agente (`sshx agent`)
* `eval $(sshx agent start)`: Inicia o SSH Agent na sessão de terminal atual.
* `sshx agent stop`: Encerra o processo do SSH Agent.
* `sshx agent add <chave>`: Adiciona uma chave do inventário ao SSH Agent.
* `sshx agent remove <chave>`: Remove uma chave específica do SSH Agent.
* `sshx agent list`: Lista as identidades ativas no SSH Agent.

---

## 🛡️ Segurança

* **Sem armazenamento de senhas**: Toda autenticação é baseada em pares de chaves SSH (recomenda-se Ed25519).
* **Controle estrito de permissões**: Os diretórios e arquivos de chaves gerados pelo SSHX recebem as permissões Unix estritas correspondentes (`0700` para pastas, `0600` para chaves privadas).
* **Sem modificação forçada**: O SSHX não modifica seus arquivos originais do OpenSSH a menos que você solicite (por exemplo, ao usar o `ssh-copy-id` interno).

---

## ⚖️ Licença

Este projeto é open-source e está licenciado sob a licença MIT.
