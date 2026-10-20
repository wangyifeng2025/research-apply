use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const UP_SQL: &str = include_str!("../sql/001_up.sql");
const DOWN_SQL: &str = include_str!("../sql/001_down.sql");

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_script(manager, UP_SQL).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        execute_script(manager, DOWN_SQL).await
    }
}

async fn execute_script(manager: &SchemaManager<'_>, script: &str) -> Result<(), DbErr> {
    let connection = manager.get_connection();
    for statement in sql_statements(script) {
        connection.execute_unprepared(statement).await?;
    }
    Ok(())
}

fn sql_statements(script: &str) -> Vec<&str> {
    script
        .split("\n-- @@")
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::sql_statements;

    #[test]
    fn up_script_covers_both_application_tracks() {
        let statements = sql_statements(include_str!("../sql/001_up.sql"));
        let joined = statements.join("\n");

        assert!(statements.len() > 20);
        assert!(joined.contains("CREATE TABLE applications"));
        assert!(joined.contains("CREATE TABLE ss_designs"));
        assert!(joined.contains("CREATE TABLE ns_indicators"));
        assert!(joined.contains("CREATE TABLE notifications"));
        assert!(joined.contains("CREATE TABLE audit_logs"));
        assert!(joined.contains("2026年院级科研项目申报"));
    }

    #[test]
    fn up_script_has_no_triggers_or_functions() {
        let script = include_str!("../sql/001_up.sql");

        assert!(!script.contains("CREATE TRIGGER"));
        assert!(!script.contains("CREATE FUNCTION"));
        assert!(!script.contains("CREATE OR REPLACE FUNCTION"));
    }

    #[test]
    fn down_script_drops_core_tables() {
        let statements = sql_statements(include_str!("../sql/001_down.sql"));
        let joined = statements.join("\n");

        assert!(joined.contains("DROP TABLE IF EXISTS applications"));
        assert!(joined.contains("DROP TABLE IF EXISTS audit_logs"));
        assert!(joined.contains("DROP TYPE IF EXISTS project_class"));
    }
}
