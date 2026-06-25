use std::path::{Path, PathBuf};

pub struct Vault {
    root: PathBuf,
}

impl Vault {
    /// Cria um vault a partir de um diretorio raiz
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Caminho completo para um arquivo .md no vault
    pub fn task_file(&self, name: &str) -> PathBuf {
        let filename = if name.ends_with(".md") {
            name.to_string()
        } else {
            format!("{}.md", name)
        };
        self.root.join(filename)
    }

    /// Caminho para o diretorio de tasks concluidas (opcional)
    pub fn done_dir(&self) -> PathBuf {
        self.root.join("done")
    }

    /// Caminho para o arquivo de coluna scrum (ex: todo.md, doing.md)
    pub fn column_file(&self, column: &str) -> PathBuf {
        let filename = format!("{}.md", column);
        self.root.join(filename)
    }

    /// Verifica se o vault existe no disco
    pub fn exists(&self) -> bool {
        self.root.exists()
    }

    /// Cria o diretorio do vault (e subdiretorios) se não existir
    pub fn ensure(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(self.done_dir())?;
        Ok(())
    }

    /// Retorna o caminho raiz do vault
    pub fn root(&self) -> &Path {
        &self.root
    }
}
