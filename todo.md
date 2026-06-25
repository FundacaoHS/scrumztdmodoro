---
id: todo
aliases:
  - TODO
  - Features
tags:
  - planning
  - project
created: 2026-06-25
updated: 2026-06-25
---

# TODO - Scrumztdmodoro

## Feature: Estrutura do Projeto
- [x] Workspace Rust (Cargo.toml raiz)
- [x] `fundacao` — lib compartilhada (config + vault paths)
- [x] `sm-core` — lib com core logic (Task, TaskList)
- [x] `sm-view` — binário `sm` com CLI + TUI + Tauri dispatch
- [x] Criar `todo.md` para controle de features
- [x] Criar `AGENTS.md` com workflow de features
- [x] Criar branch `develop` local e remota

## Feature: fundacao-crate
- [x] Criar crate `fundacao` com `Config` (builder + load/save YAML)
- [x] Criar `Vault` para gerenciar paths de diretorio .md
- [x] Adicionar `fundacao` como dependencia do `sm-core`
- [x] Compilar e verificar integração

## Feature: CLI Basica
- [x] `sm add --task "..." --tags "..."`
- [x] `sm remove --id <id>`
- [x] `sm toggle --id <id>`
- [x] `sm list`

## Feature: TUI Basica
- [x] Navegação ↑↓/jk
- [x] Toggle task (Space/Enter)
- [x] Deletar task (d)
- [x] Sair (q)

## Feature: Config - Add & Exists
- [x] Adicionar enum `ConfigKey` com variante `Vault`
- [x] Adicionar struct `VaultConfig` (scaffold)
- [x] Implementar `Config::add(key, valor)` esboço inicial
- [x] Implementar `Config::exists()` (privado)
- [x] Escrever testes iniciais
- [x] `add()` chamar `exists()` internamente e criar vault + .conf se não existir
- [x] Adicionar `AddStatus` (Created / AlreadyExists / NoChange)
- [x] Atualizar testes para o novo comportamento
- [x] Adicionar variante `Column(Vec<String>)`, `Todo(TodoConfig)`, `Project(ProjectConfig)`
