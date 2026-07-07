# SSHX CLI Reference

> **SSHX** é um gerenciador moderno de conexões SSH que abstrai a complexidade do OpenSSH, oferecendo uma experiência simples, consistente e intuitiva.

---

# Comandos

## `sshx add`

Adiciona um novo servidor ao SSHX.

### Sintaxe

```bash
sshx add <nome>
```

### Exemplo

```bash
sshx add producao
```

### O que faz

- Solicita host, usuário e porta.
- Gera uma chave SSH (caso necessário).
- Instala automaticamente a chave pública no servidor.
- Salva a configuração local.

### Equivalente SSH

```bash
ssh-keygen -t ed25519
ssh-copy-id usuario@host
# editar ~/.ssh/config
```

---

## `sshx connect`

Conecta-se a um servidor.

### Sintaxe

```bash
sshx connect <nome>
```

ou

```bash
sshx <nome>
```

### Exemplo

```bash
sshx producao
```

### Equivalente SSH

```bash
ssh usuario@host
```

ou

```bash
ssh -i ~/.ssh/minha-chave usuario@host
```

---

## `sshx list`

Lista todos os servidores cadastrados.

### Sintaxe

```bash
sshx list
```

### Alias

```bash
sshx ls
```

### Equivalente SSH

Não existe.

---

## `sshx info`

Exibe informações detalhadas sobre um servidor.

### Sintaxe

```bash
sshx info <nome>
```

### Exemplo

```bash
sshx info producao
```

### Informações

- Host
- Usuário
- Porta
- Fingerprint da chave
- Data de criação
- Última conexão
- Status da conexão

### Equivalente SSH

```bash
cat ~/.ssh/config
cat ~/.ssh/known_hosts
ssh-keygen -lf ~/.ssh/id_ed25519.pub
```

---

## `sshx remove`

Remove um servidor da configuração local.

### Sintaxe

```bash
sshx remove <nome>
```

### Exemplo

```bash
sshx remove producao
```

### Equivalente SSH

Editar manualmente:

```text
~/.ssh/config
```

---

## `sshx rename`

Renomeia um servidor.

### Sintaxe

```bash
sshx rename <atual> <novo>
```

### Exemplo

```bash
sshx rename prod producao
```

### Equivalente SSH

Editar manualmente o arquivo:

```text
~/.ssh/config
```

---

## `sshx copy`

Copia arquivos para ou do servidor.

### Enviar

```bash
sshx copy arquivo.zip producao:/opt/app
```

### Receber

```bash
sshx copy producao:/var/log/app.log .
```

### Equivalente SSH

```bash
scp arquivo.zip usuario@host:/opt/app

scp usuario@host:/var/log/app.log .
```

---

## `sshx exec`

Executa um comando remoto.

### Sintaxe

```bash
sshx exec <servidor> "<comando>"
```

### Exemplo

```bash
sshx exec producao "docker ps"
```

### Equivalente SSH

```bash
ssh producao "docker ps"
```

---

## `sshx shell`

Abre um terminal remoto.

### Sintaxe

```bash
sshx shell <servidor>
```

### Equivalente SSH

```bash
ssh usuario@host
```

---

## `sshx install-key`

Instala uma chave pública em um servidor.

### Sintaxe

```bash
sshx install-key <servidor>
```

### Exemplo

```bash
sshx install-key producao
```

### Equivalente SSH

```bash
ssh-copy-id usuario@host
```

---

## `sshx keys`

Lista todas as chaves cadastradas.

### Sintaxe

```bash
sshx keys
```

### Exibe

- Nome
- Tipo
- Fingerprint
- Caminho
- Quantos servidores utilizam

### Equivalente SSH

```bash
ls ~/.ssh
```

---

## `sshx key create`

Cria uma nova chave SSH.

### Sintaxe

```bash
sshx key create <nome>
```

### Exemplo

```bash
sshx key create github
```

### Equivalente SSH

```bash
ssh-keygen -t ed25519
```

---

## `sshx key delete`

Remove uma chave.

### Sintaxe

```bash
sshx key delete <nome>
```

### Equivalente SSH

```bash
rm ~/.ssh/id_ed25519
rm ~/.ssh/id_ed25519.pub
```

---

## `sshx key rotate`

Gera uma nova chave e substitui automaticamente a antiga no servidor.

### Sintaxe

```bash
sshx key rotate <servidor>
```

### Equivalente SSH

Não existe um comando único.

Envolve:

- gerar nova chave;
- copiar chave pública;
- remover chave antiga;
- atualizar configuração.

---

## `sshx sync`

Sincroniza uma chave com vários servidores.

### Sintaxe

```bash
sshx sync
```

### O que faz

- Detecta servidores cadastrados.
- Instala ou atualiza automaticamente as chaves públicas.

### Equivalente SSH

Não existe.

---

## `sshx doctor`

Analisa problemas na configuração.

### Sintaxe

```bash
sshx doctor
```

### Verificações

- Permissões incorretas
- Chaves inexistentes
- Hosts inacessíveis
- Fingerprints alterados
- Configuração inválida
- Conectividade

### Equivalente SSH

Diversos comandos manuais:

```bash
ssh -vvv
chmod
ssh-keygen
```

---

## `sshx ping`

Verifica se um servidor está acessível.

### Sintaxe

```bash
sshx ping <servidor>
```

### Exemplo

```bash
sshx ping producao
```

### Equivalente SSH

Não existe diretamente.

Normalmente seria:

```bash
ping host
```

ou

```bash
ssh usuario@host exit
```

---

## `sshx config`

Exibe a configuração do servidor.

### Sintaxe

```bash
sshx config <servidor>
```

### Equivalente SSH

```bash
cat ~/.ssh/config
```

---

## `sshx export`

Exporta toda a configuração do SSHX.

### Sintaxe

```bash
sshx export
```

### Formatos

- YAML
- JSON
- TOML

### Equivalente SSH

Não existe.

---

## `sshx import`

Importa configurações.

### Sintaxe

```bash
sshx import servidores.yaml
```

### Equivalente SSH

Não existe.

---

## `sshx agent start`

Inicia o SSH Agent.

### Sintaxe

```bash
sshx agent start
```

### Equivalente SSH

```bash
eval "$(ssh-agent -s)"
```

---

## `sshx agent stop`

Finaliza o SSH Agent.

### Equivalente SSH

```bash
killall ssh-agent
```

---

## `sshx agent add`

Adiciona uma chave ao Agent.

### Sintaxe

```bash
sshx agent add github
```

### Equivalente SSH

```bash
ssh-add ~/.ssh/github
```

---

## `sshx agent remove`

Remove uma chave do Agent.

### Equivalente SSH

```bash
ssh-add -d ~/.ssh/github
```

---

## `sshx agent list`

Lista as chaves carregadas no Agent.

### Equivalente SSH

```bash
ssh-add -l
```

---

# Estrutura sugerida

```
~/.config/sshx/

servers.yaml
keys.yaml
profiles.yaml
history.db
known_hosts

keys/
├── github
├── producao
├── homelab
└── oracle
```

---

# Filosofia

O SSHX não substitui o OpenSSH.

Ele atua como uma camada de abstração sobre ferramentas consolidadas como:

- ssh
- scp
- ssh-agent
- ssh-keygen
- ssh-copy-id

Seu objetivo é transformar tarefas compostas por diversos comandos em uma única operação simples, mantendo compatibilidade total com a infraestrutura SSH existente.