# SSHX Features

> SSHX é um gerenciador moderno de SSH que simplifica a administração de servidores sem substituir o OpenSSH. Seu objetivo é eliminar tarefas repetitivas, automatizar configurações e oferecer uma experiência consistente em Linux, macOS e Windows.

---

# Experiência do Desenvolvedor

## Interface intuitiva

Substitui comandos longos e complexos por comandos simples e memoráveis.

**Antes**

```bash
ssh -i ~/.ssh/producao root@152.xxx.xxx.xxx
```

**Depois**

```bash
sshx producao
```

---

## Curva de aprendizado reduzida

Não é necessário conhecer toda a sintaxe do OpenSSH para utilizar recursos avançados.

---

## Compatibilidade total

Funciona sobre o OpenSSH existente, mantendo compatibilidade com qualquer servidor SSH.

---

## Multiplataforma

Suporte nativo para:

- Linux
- macOS
- Windows

---

## Binário único

Não requer instalação de runtimes ou dependências externas.

---

## Autocomplete

Suporte para:

- Bash
- ZSH
- Fish
- PowerShell

---

# Gerenciamento de Servidores

## Cadastro simplificado

Adicione servidores através de um assistente interativo.

---

## Organização por nomes

Conecte-se utilizando nomes amigáveis em vez de IPs.

Exemplo:

```text
producao
staging
oracle
homelab
```

---

## Organização por ambientes

Agrupe servidores por categorias.

Exemplo:

- Produção
- Desenvolvimento
- Homelab
- Clientes
- Cloud

---

## Perfis

Permite criar diferentes perfis para ambientes pessoais, profissionais ou clientes.

---

## Histórico de conexões

Registro automático das últimas conexões realizadas.

---

## Informações centralizadas

Visualização rápida de:

- Host
- Usuário
- Porta
- Chave utilizada
- Última conexão
- Status

---

# Gerenciamento de Chaves SSH

## Criação automática de chaves

Geração de chaves modernas utilizando Ed25519.

---

## Instalação automática

Instala a chave pública no servidor sem necessidade de edição manual.

---

## Rotação de chaves

Substituição segura de chaves antigas por novas.

---

## Inventário de chaves

Lista todas as chaves cadastradas e seus respectivos servidores.

---

## Compartilhamento inteligente

Uma mesma chave pode ser utilizada por múltiplos servidores quando desejado.

---

## Revogação simples

Remoção rápida de chaves comprometidas.

---

# SSH Agent

## Integração automática

Gerenciamento transparente do SSH Agent.

---

## Carregamento automático

Carrega chaves conforme necessário.

---

## Administração simplificada

Adicionar, remover e listar chaves com poucos comandos.

---

# Transferência de Arquivos

## SCP simplificado

Upload e download utilizando nomes de servidores.

---

## SFTP integrado

Acesso facilitado a servidores utilizando a mesma configuração do SSHX.

---

# Automação

## Execução remota

Executa comandos em servidores sem abrir um terminal interativo.

---

## Sincronização de chaves

Distribuição automática de chaves públicas entre vários servidores.

---

## Provisionamento inicial

Preparação automática de novos servidores.

Inclui:

- Instalação de chaves
- Configuração inicial
- Ajustes básicos de segurança

---

# Diagnóstico

## Doctor

Análise completa da configuração SSH.

Verifica automaticamente:

- Permissões incorretas
- Chaves ausentes
- Hosts inacessíveis
- Configurações inválidas
- Problemas de autenticação
- Fingerprints alterados

---

## Verificação de conectividade

Teste rápido de disponibilidade dos servidores cadastrados.

---

## Auditoria

Identificação de:

- Chaves não utilizadas
- Configurações inseguras
- Servidores inacessíveis
- Chaves duplicadas

---

# Configuração

## Exportação

Exporta toda a configuração para:

- YAML
- JSON
- TOML

---

## Importação

Importa rapidamente ambientes completos.

---

## Backup

Facilita o backup de toda a configuração.

---

## Portabilidade

Migração simples entre computadores.

---

# Segurança

## Compatível com OpenSSH

Utiliza todos os mecanismos de segurança já consolidados.

---

## Suporte a Ed25519

Utiliza algoritmos modernos de criptografia.

---

## Nunca armazena senhas

Toda autenticação é baseada em chaves SSH.

---

## Fingerprints

Validação automática da identidade dos servidores.

---

## Gerenciamento seguro das chaves

Organização centralizada das identidades SSH.

---

# Produtividade

## Menos comandos

Redução significativa da quantidade de comandos necessários para tarefas comuns.

---

## Menos configuração manual

Elimina a necessidade de editar arquivos como:

- `~/.ssh/config`
- `authorized_keys`

na maioria dos casos.

---

## Fluxo consistente

A mesma experiência independentemente do sistema operacional.

---

## Foco no trabalho

O usuário trabalha com servidores, não com configurações SSH.

---

# Arquitetura

## Baseado no OpenSSH

Não substitui o OpenSSH.

Aproveita toda a estabilidade, segurança e compatibilidade do ecossistema existente.

---

## Extensível

Arquitetura preparada para novos recursos através de módulos e plugins.

---

## Open Source

Projeto aberto para auditoria, colaboração e evolução pela comunidade.

---

# Filosofia

O SSHX existe para tornar o SSH simples.

Enquanto o OpenSSH fornece o mecanismo de comunicação segura, o SSHX oferece uma experiência moderna, intuitiva e produtiva para administrar conexões, servidores e identidades SSH.