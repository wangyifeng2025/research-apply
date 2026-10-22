use entity::audit_logs;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, DbErr};
use serde_json::Value;
use uuid::Uuid;

pub struct AuditEntry {
    pub actor_id: Option<Uuid>,
    pub action: &'static str,
    pub target_type: &'static str,
    pub target_id: Option<Uuid>,
    pub detail: Option<Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

pub async fn record<C>(connection: &C, entry: AuditEntry) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    let model = audit_logs::ActiveModel {
        actor_id: Set(entry.actor_id),
        action: Set(entry.action.to_owned()),
        target_type: Set(entry.target_type.to_owned()),
        target_id: Set(entry.target_id),
        detail: Set(entry.detail),
        ip_address: Set(entry.ip_address),
        user_agent: Set(entry.user_agent),
        created_at: Set(super::time::now()),
        ..Default::default()
    };
    model.insert(connection).await?;
    Ok(())
}
