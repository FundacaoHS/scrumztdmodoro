use chrono::Local;
use std::fs;
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

    /// Arquivo todo do dia (ex: 2026-06-25.md)
    pub fn today_todo(&self) -> PathBuf {
        let date = Local::now().format("%Y-%m-%d").to_string();
        self.task_file(&date)
    }

    /// Caminho para o diretorio de notas
    pub fn notes_dir(&self) -> PathBuf {
        self.root.join("notes")
    }

    /// Arquivo de notas de uma data especifica
    pub fn notes_file_for(&self, date: &str) -> PathBuf {
        let filename = format!("{}.md", date);
        self.notes_dir().join(filename)
    }

    /// Arquivo de notas do dia de hoje
    pub fn notes_file(&self) -> PathBuf {
        let date = Local::now().format("%Y-%m-%d").to_string();
        self.notes_file_for(&date)
    }

    /// Caminho do arquivo .md mais recente antes de hoje, se existir
    pub fn last_day_path(&self) -> Option<PathBuf> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let mut entries: Vec<PathBuf> = fs::read_dir(&self.root)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
                    .filter(|p| {
                        p.extension().is_some_and(|ext| ext == "md")
                            && p.file_stem().is_some_and(|s| {
                                let name = s.to_string_lossy();
                                name.len() == 10 && name.as_ref() < today.as_str()
                            })
                    })
            .collect();
        entries.sort();
        entries.last().cloned()
    }

    /// Lista todos arquivos .md no root ordenados por nome
    pub fn all_md_files(&self) -> Vec<PathBuf> {
        let mut entries: Vec<PathBuf> = fs::read_dir(&self.root)
            .ok()
            .into_iter()
            .flat_map(|r| r.filter_map(|e| e.ok()))
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
            .collect();
        entries.sort();
        entries
    }

    /// Caminho para o diretorio de tasks concluidas (opcional)
    pub fn done_dir(&self) -> PathBuf {
        self.root.join("done")
    }

    /// Caminho para o arquivo de coluna scrum (ex: doing.md)
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
        std::fs::create_dir_all(self.notes_dir())?;
        Ok(())
    }

    /// Retorna o caminho raiz do vault
    pub fn root(&self) -> &Path {
        &self.root
    }
}
