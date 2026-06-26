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

## Feature: Task - Bullet Journal
- [x] Adicionar enum `BulletKind` (Task, Done, Migrated, Scheduled, Event, Note, Priority)
- [x] Adicionar campo `bullet` na struct `Task`
- [x] Implementar parser de bullet de/para markdown
- [x] Implementar `TaskList::from_md(text)` — ler tasks do .md
- [x] Implementar `TaskList::to_md(&self)` — escrever tasks como .md
- [x] Conectar com `fundacao::Vault` para ler/escrever `today_todo()`
- [x] Escrever testes

## Feature: Pomodoro Timer
- [x] Criar struct `SessionState` (Idle, Focusing, ShortBreak, LongBreak)
- [x] Criar struct `Pomodoro` com config (focus, short, long, cycles) + estado + timer
- [x] Implementar persistência de sessão ativa e histórico no vault
- [x] Implementar comandos CLI (`sm pomo start/status/stop/config`)
- [x] Integrar timer view na TUI
- [x] Escrever testes

## Feature: Backlog + Migração Automática + Notas
- [ ] vault.rs: notes_dir, notes_file, last_day_path
- [ ] task.rs: helpers de marcador [backlog]/[todo]/[cancelled]
- [ ] task.rs: add_backlog, add_today, list_daily, list_backlog
- [ ] task.rs: pull_task, pull_all, mark_cancelled
- [ ] task.rs: migrate_untouched — tasks inacabadas → [todo] no hoje + notas → notes/
- [ ] Testes de backlog e migrate
- [ ] cli.rs: Backlog, Pull, Cancel, Add --today
- [ ] main.rs: trigger migração automática no init()
- [ ] tui.rs: backlog view (tecla B), notes view (N), cancelar (c)
- [ ] tui.rs: título mostra daily/backlog count

## Feature: sm-tauri — GUI Desktop (concluída)
- [x] Criar crate `sm-tauri` como workspace member
- [x] Adicionar dependências: `tauri`, `sm-core`, `fundacao`
- [x] Implementar Tauri commands CRUD (list, add, remove, toggle, set_bullet)
- [x] Implementar comandos Pomodoro (start, stop, status)
- [x] Criar `tauri.conf.json` + `capabilities/default.json`
- [x] Criar `index.html` com frontend básico JS
- [x] Compilar com sucesso

## Feature: Neovim Plugin
- [ ] Plugin Lua que invoca `sm` CLI para integrar Bullet Journal + Pomodoro no Neovim
- [ ] Comando `:SMTasks` — lista tasks do dia num buffer
- [ ] Comando `:SMAdd "desc" #tag` — adiciona task
- [ ] Comando `:SMToggle` / `:SMDelete` sob cursor
- [ ] Comando `:SMPomo start|stop|status` — timer Pomodoro no statusline
- [ ] Highlight de bullets (cores diferentes por tipo)
- [ ] Atualização automática ao salvar buffer
- [ ] **Scanner de TODOs com fzf**: buscar `- [ ]`, `TODO:`, `FIXME:`, `HACK:`, `XXX:` nos arquivos do projeto via `rg` + `fzf` para seleção interativa
- [ ] Pular para a linha do arquivo ao selecionar um TODO escaneado
- [ ] Opção de importar um TODO escaneado como task do Bullet Journal (`:SMImport`)
- [ ] Adicionar highlight customizado para TODOs nos buffers (signcolumn + virtual text)
- [ ] Estruturar como plugin standalone (`sm.nvim`) com `lua/sm/init.lua`
- [ ] Usar `vim.system()` ou `jobstart` para chamar `sm` CLI assincronamente
