use chrono::TimeZone;
use entity::application_cycles;
use entity::departments;
use entity::sea_orm_active_enums::{CycleStatus, UserRole};
use entity::user_roles;
use entity::users;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::auth::hash_password;
use crate::config::Config;
use crate::error::AppError;
use crate::state::connect_database;
use crate::support::time::now;

struct DemoUser {
    username: &'static str,
    password: &'static str,
    real_name: &'static str,
    employee_no: &'static str,
    department: &'static str,
    role: UserRole,
    gender: Option<&'static str>,
    professional_title: Option<&'static str>,
    research_expertise: Option<&'static str>,
    education: Option<&'static str>,
    degree: Option<&'static str>,
    phone: Option<&'static str>,
}

const USERS: &[DemoUser] = &[
    DemoUser {
        username: "zhaomingyuan",
        password: "research2026",
        real_name: "赵明远",
        employee_no: "T2026001",
        department: "EM",
        role: UserRole::Teacher,
        gender: Some("男"),
        professional_title: Some("副教授"),
        research_expertise: Some("区域经济"),
        education: Some("研究生"),
        degree: Some("硕士"),
        phone: Some("13700001001"),
    },
    DemoUser {
        username: "chenxia",
        password: "research2026",
        real_name: "陈霞",
        employee_no: "T2026002",
        department: "IE",
        role: UserRole::Teacher,
        gender: Some("女"),
        professional_title: Some("讲师"),
        research_expertise: Some("计算机应用"),
        education: Some("本科"),
        degree: Some("学士"),
        phone: Some("13700001002"),
    },
    DemoUser {
        username: "liwenbo",
        password: "dept2026",
        real_name: "李文博",
        employee_no: "D2026001",
        department: "EM",
        role: UserRole::DepartmentReviewer,
        gender: None,
        professional_title: Some("教授"),
        research_expertise: Some("区域经济"),
        education: None,
        degree: None,
        phone: None,
    },
    DemoUser {
        username: "tangwei",
        password: "dept2026",
        real_name: "唐伟",
        employee_no: "D2026002",
        department: "IE",
        role: UserRole::DepartmentReviewer,
        gender: None,
        professional_title: Some("副教授"),
        research_expertise: Some("计算机应用"),
        education: None,
        degree: None,
        phone: None,
    },
    DemoUser {
        username: "wanghaining",
        password: "office2026",
        real_name: "王海宁",
        employee_no: "O2026001",
        department: "RO",
        role: UserRole::ResearchOffice,
        gender: None,
        professional_title: None,
        research_expertise: None,
        education: None,
        degree: None,
        phone: None,
    },
    DemoUser {
        username: "expert-e016",
        password: "review2026",
        real_name: "匿名专家E-016",
        employee_no: "E2026016",
        department: "RO",
        role: UserRole::Expert,
        gender: None,
        professional_title: Some("教授"),
        research_expertise: Some("区域经济"),
        education: None,
        degree: None,
        phone: None,
    },
    DemoUser {
        username: "expert-e017",
        password: "review2026",
        real_name: "匿名专家E-017",
        employee_no: "E2026017",
        department: "RO",
        role: UserRole::Expert,
        gender: None,
        professional_title: Some("教授"),
        research_expertise: Some("计算机应用"),
        education: None,
        degree: None,
        phone: None,
    },
    DemoUser {
        username: "zhoujin",
        password: "admin2026",
        real_name: "周谨",
        employee_no: "A2026001",
        department: "RO",
        role: UserRole::Admin,
        gender: None,
        professional_title: None,
        research_expertise: None,
        education: None,
        degree: None,
        phone: None,
    },
];

pub async fn run() -> Result<(), AppError> {
    let database_url = Config::database_only()?;
    let db = connect_database(&database_url).await?;
    let economy = ensure_department(&db, "经济管理学院", "EM").await?;
    let information = ensure_department(&db, "信息工程学院", "IE").await?;
    let office = ensure_department(&db, "科研处", "RO").await?;
    let departments = [("EM", economy), ("IE", information), ("RO", office)];

    for demo in USERS {
        let department_id = departments
            .iter()
            .find(|(code, _)| *code == demo.department)
            .map(|(_, id)| *id);
        ensure_user(&db, demo, department_id).await?;
        println!(
            "{}  {}  密码 {}",
            role_label(&demo.role),
            demo.username,
            demo.password
        );
    }
    open_cycle(&db).await?;
    println!("2026年院级科研项目申报 已开放");
    println!("以上为测试账号。正式使用前必须更换全部密码，不要把密码写在登录页。");
    Ok(())
}

async fn ensure_department(
    db: &DatabaseConnection,
    name: &str,
    code: &str,
) -> Result<Uuid, AppError> {
    if let Some(existing) = departments::Entity::find()
        .filter(departments::Column::Code.eq(code))
        .one(db)
        .await?
    {
        return Ok(existing.id);
    }
    let id = Uuid::new_v4();
    let timestamp = now();
    departments::ActiveModel {
        id: Set(id),
        name: Set(name.to_owned()),
        code: Set(Some(code.to_owned())),
        created_at: Set(timestamp),
        updated_at: Set(timestamp),
    }
    .insert(db)
    .await?;
    Ok(id)
}

async fn ensure_user(
    db: &DatabaseConnection,
    demo: &DemoUser,
    department_id: Option<Uuid>,
) -> Result<(), AppError> {
    let password_hash = hash_password(demo.password)?;
    let timestamp = now();
    let user_id = if let Some(existing) = users::Entity::find()
        .filter(users::Column::Username.eq(demo.username))
        .one(db)
        .await?
    {
        let mut model: users::ActiveModel = existing.clone().into();
        model.password_hash = Set(password_hash);
        model.real_name = Set(demo.real_name.to_owned());
        model.employee_no = Set(Some(demo.employee_no.to_owned()));
        model.department_id = Set(department_id);
        model.gender = Set(owned(demo.gender));
        model.professional_title = Set(owned(demo.professional_title));
        model.research_expertise = Set(owned(demo.research_expertise));
        model.education = Set(owned(demo.education));
        model.degree = Set(owned(demo.degree));
        model.phone = Set(owned(demo.phone));
        model.is_active = Set(true);
        model.updated_at = Set(timestamp);
        model.update(db).await?;
        existing.id
    } else {
        let id = Uuid::new_v4();
        users::ActiveModel {
            id: Set(id),
            username: Set(demo.username.to_owned()),
            password_hash: Set(password_hash),
            real_name: Set(demo.real_name.to_owned()),
            employee_no: Set(Some(demo.employee_no.to_owned())),
            department_id: Set(department_id),
            gender: Set(owned(demo.gender)),
            professional_title: Set(owned(demo.professional_title)),
            research_expertise: Set(owned(demo.research_expertise)),
            education: Set(owned(demo.education)),
            degree: Set(owned(demo.degree)),
            phone: Set(owned(demo.phone)),
            is_active: Set(true),
            created_at: Set(timestamp),
            updated_at: Set(timestamp),
            ..Default::default()
        }
        .insert(db)
        .await?;
        id
    };

    let already = user_roles::Entity::find_by_id((user_id, demo.role.clone()))
        .one(db)
        .await?;
    if already.is_none() {
        user_roles::ActiveModel {
            user_id: Set(user_id),
            role: Set(demo.role.clone()),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

async fn open_cycle(db: &DatabaseConnection) -> Result<(), AppError> {
    let Some(cycle) = application_cycles::Entity::find()
        .filter(application_cycles::Column::CycleYear.eq(2026))
        .one(db)
        .await?
    else {
        return Err(AppError::NotFound(
            "没有找到 2026 年申报批次，请先执行数据库迁移".to_owned(),
        ));
    };
    let mut model: application_cycles::ActiveModel = cycle.into();
    model.status = Set(CycleStatus::Open);
    model.opens_at = Set(Some(shanghai(2026, 1, 1)?));
    model.closes_at = Set(Some(shanghai(2026, 12, 31)?));
    model.updated_at = Set(now());
    model.update(db).await?;
    Ok(())
}

fn shanghai(
    year: i32,
    month: u32,
    day: u32,
) -> Result<chrono::DateTime<chrono::FixedOffset>, AppError> {
    let offset = chrono::FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| AppError::Internal("时区无效".to_owned()))?;
    offset
        .with_ymd_and_hms(year, month, day, 0, 0, 0)
        .single()
        .ok_or_else(|| AppError::Internal("批次时间无效".to_owned()))
}

fn owned(value: Option<&str>) -> Option<String> {
    value.map(str::to_owned)
}

fn role_label(role: &UserRole) -> &'static str {
    match role {
        UserRole::Teacher => "教师    ",
        UserRole::DepartmentReviewer => "系审核人",
        UserRole::ResearchOffice => "科研处  ",
        UserRole::Expert => "专家    ",
        UserRole::Admin => "管理员  ",
    }
}
