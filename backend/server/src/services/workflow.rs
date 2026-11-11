//! 申报状态机。合法边与需求 4.1 一致；真正改状态时带上旧状态条件。
//! 尚未有专家提交时，撤销全部指派会从盲审中回到科研处已通过。

use entity::applications;
use entity::sea_orm_active_enums::ApplicationStatus;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveEnum, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::error::AppError;
use crate::support::time::now;

pub fn allows(from: ApplicationStatus, to: ApplicationStatus) -> bool {
    matches!(
        (from, to),
        (ApplicationStatus::Draft, ApplicationStatus::Submitted)
            | (ApplicationStatus::Submitted, ApplicationStatus::Draft)
            | (
                ApplicationStatus::Submitted,
                ApplicationStatus::DeptApproved
            )
            | (
                ApplicationStatus::Submitted,
                ApplicationStatus::DeptReturned
            )
            | (
                ApplicationStatus::Submitted,
                ApplicationStatus::DeptRejected
            )
            | (
                ApplicationStatus::DeptReturned,
                ApplicationStatus::Submitted
            )
            | (
                ApplicationStatus::DeptApproved,
                ApplicationStatus::OfficeApproved
            )
            | (
                ApplicationStatus::DeptApproved,
                ApplicationStatus::OfficeReturned
            )
            | (
                ApplicationStatus::DeptApproved,
                ApplicationStatus::OfficeRejected
            )
            | (
                ApplicationStatus::OfficeReturned,
                ApplicationStatus::Submitted
            )
            | (
                ApplicationStatus::OfficeApproved,
                ApplicationStatus::ExpertReviewing
            )
            | (
                ApplicationStatus::ExpertReviewing,
                ApplicationStatus::OfficeApproved
            )
            | (
                ApplicationStatus::ExpertReviewing,
                ApplicationStatus::Established
            )
            | (
                ApplicationStatus::ExpertReviewing,
                ApplicationStatus::ExpertRejected
            )
    )
}

/// 只在 `from` 仍是当前状态时改成 `to`。影响 0 行表示别人已经改过，返回 409。
pub async fn transit<C>(
    connection: &C,
    id: Uuid,
    from: ApplicationStatus,
    to: ApplicationStatus,
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    if !allows(from.clone(), to.clone()) {
        return Err(AppError::Conflict("当前状态不能这样流转".to_owned()));
    }
    let result = applications::Entity::update_many()
        .col_expr(applications::Column::Status, Expr::value(to.to_value()))
        .col_expr(applications::Column::UpdatedAt, Expr::value(now()))
        .filter(applications::Column::Id.eq(id))
        .filter(applications::Column::Status.eq(from))
        .exec(connection)
        .await?;
    if result.rows_affected == 0 {
        return Err(AppError::Conflict(
            "申报书状态已变化，请刷新后重试".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::allows;
    use entity::sea_orm_active_enums::ApplicationStatus::{
        DeptApproved, DeptRejected, DeptReturned, Draft, Established, ExpertRejected,
        ExpertReviewing, OfficeApproved, OfficeRejected, OfficeReturned, Submitted,
    };

    #[test]
    fn allows_every_edge_in_the_status_graph() {
        let legal = [
            (Draft, Submitted),
            (Submitted, Draft),
            (Submitted, DeptApproved),
            (Submitted, DeptReturned),
            (Submitted, DeptRejected),
            (DeptReturned, Submitted),
            (DeptApproved, OfficeApproved),
            (DeptApproved, OfficeReturned),
            (DeptApproved, OfficeRejected),
            (OfficeReturned, Submitted),
            (OfficeApproved, ExpertReviewing),
            (ExpertReviewing, OfficeApproved),
            (ExpertReviewing, Established),
            (ExpertReviewing, ExpertRejected),
        ];
        for (from, to) in legal {
            assert!(
                allows(from.clone(), to.clone()),
                "{from:?} -> {to:?} 应该允许"
            );
        }
    }

    #[test]
    fn rejects_edges_outside_the_status_graph() {
        let illegal = [
            (Submitted, Established),
            (Draft, DeptApproved),
            (DeptApproved, ExpertReviewing),
            (DeptRejected, Submitted),
            (OfficeRejected, Draft),
            (ExpertRejected, Established),
            (Established, Draft),
            (OfficeApproved, Established),
            (Submitted, OfficeApproved),
            (OfficeApproved, DeptApproved),
            (Draft, Draft),
        ];
        for (from, to) in illegal {
            assert!(
                !allows(from.clone(), to.clone()),
                "{from:?} -> {to:?} 不应该允许"
            );
        }
    }
}
