# Regras do Projeto

## Workflow de Features

1. **Antes de começar**: atualizar `todo.md` com a feature e suas tasks
2. **Branch**: criar branch local + remota a partir de `develop`
   - Nome: `feature/<nome-da-feature>`
3. **Desenvolvimento**: trabalhar na feature até completar todas as tasks
4. **Commits**: um commit por task concluida
   - Formato: `feat: <descrição curta da task>`
5. **Finalização**: ao terminar todas as tasks da feature:
   - Executar `cargo test` (ou comando de teste equivalente)
   - Se os testes passarem: merge para `develop`
   - Deletar branch local e remota da feature
   - Atualizar `todo.md` marcando a feature como concluida
