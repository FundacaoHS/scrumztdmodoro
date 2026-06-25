use fundacao::{Config, ConfigKey, ProjectConfig, TodoConfig, VaultConfig};

fn main() {
    use fundacao::Vault;

    let cfg = Config::builder()
        .vault("./meu_vault")
        .project_name("projeto-x")
        .scrum_columns(vec![
            "backlog".into(),
            "todo".into(),
            "doing".into(),
            "done".into(),
        ])
        .build();

    println!("=== Builder ===");
    println!("{}", serde_yaml::to_string(&cfg).unwrap());

    println!("\n=== add(Project) + add(Todo) ===");
    let mut cfg2 = Config::default();
    cfg2.add(ConfigKey::Vault(VaultConfig::new().path("./vault_x")))
        .unwrap();
    cfg2.add(ConfigKey::Project(ProjectConfig::new().name("proj-a")))
        .unwrap();
    cfg2.add(ConfigKey::Todo(TodoConfig::new().file_name("tasks.md")))
        .unwrap();
    println!("{}", serde_yaml::to_string(&cfg2).unwrap());

    println!("\n=== Vault::today_todo() ===");
    let vault = Vault::new("./meu_vault");
    println!("Hoje: {}", vault.today_todo().display());

    cfg2.save("./sm-example.yaml").unwrap();
    println!("\nSalvo em ./sm-example.yaml");
}
