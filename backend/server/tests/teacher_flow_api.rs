mod common;

use axum::http::{Method, StatusCode};
use common::{PASSWORD, call, call_bytes, login, with_test_app};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
#[ignore = "需要本地 PostgreSQL，使用独立测试库"]
async fn auth_covers_login_unauthorized_and_forbidden() {
    with_test_app(|app| async move {
        let (status, body, token) = login(&app.app, "flow-teacher", "wrong-password").await?;
        if status != StatusCode::UNAUTHORIZED || token.is_some() {
            return Err(format!("错误密码应返回 401，实际 {status} {body}"));
        }
        let (status, body, _) = login(&app.app, "flow-inactive", PASSWORD).await?;
        if status != StatusCode::FORBIDDEN {
            return Err(format!("停用账号应返回 403，实际 {status} {body}"));
        }
        let (status, _) = call(&app.app, Method::GET, "/api/auth/me", None, None).await?;
        if status != StatusCode::UNAUTHORIZED {
            return Err(format!("未登录访问应返回 401，实际 {status}"));
        }
        let (status, _) = call(
            &app.app,
            Method::GET,
            "/api/admin/users",
            Some(&app.teacher),
            None,
        )
        .await?;
        if status != StatusCode::FORBIDDEN {
            return Err(format!("教师访问用户管理应返回 403，实际 {status}"));
        }
        let (status, body) = call(
            &app.app,
            Method::GET,
            "/api/auth/me",
            Some(&app.teacher),
            None,
        )
        .await?;
        if status != StatusCode::OK || body["user"]["username"] != "flow-teacher" {
            return Err(format!("登录后的会话不能读取当前用户: {status} {body}"));
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "需要本地 PostgreSQL，使用独立测试库"]
async fn teacher_submits_social_and_natural_and_cannot_touch_others() {
    with_test_app(|app| async move {
        let (status, _) = call(
            &app.app,
            Method::POST,
            "/api/applications",
            Some(&app.office),
            Some(json!({ "project_class": "social" })),
        )
        .await?;
        if status != StatusCode::FORBIDDEN {
            return Err(format!("科研处新建申报应返回 403，实际 {status}"));
        }

        let social = create_application(&app.app, &app.teacher, "social").await?;
        if social["principal_name"] != "流程教师"
            || social["professional_title"] != "副教授"
            || social["department_name"] != "接口测试学院"
            || social["mobile_phone"] != "13700001001"
        {
            return Err(format!("新建申报没有带出个人资料: {social}"));
        }
        let social_id = social["id"].as_str().ok_or("申报书缺少 id")?;
        expect_status(
            &app.app,
            Method::GET,
            &format!("/api/applications/{social_id}"),
            Some(&app.other),
            None,
            StatusCode::FORBIDDEN,
        )
        .await?;
        expect_status(
            &app.app,
            Method::PUT,
            &format!("/api/applications/{social_id}/basic"),
            Some(&app.other),
            Some(basic_body("越权修改")),
            StatusCode::FORBIDDEN,
        )
        .await?;
        expect_status(
            &app.app,
            Method::DELETE,
            &format!("/api/applications/{social_id}"),
            Some(&app.other),
            None,
            StatusCode::FORBIDDEN,
        )
        .await?;

        save_basic(&app.app, &app.teacher, social_id, "县域产业研究").await?;
        save_budget(&app.app, &app.teacher, social_id, "ss_books", "1.20").await?;
        expect_status(
            &app.app,
            Method::POST,
            &format!("/api/applications/{social_id}/submit"),
            Some(&app.teacher),
            Some(json!({ "commitment_accepted": false })),
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .await?;
        let submitted = expect_status(
            &app.app,
            Method::POST,
            &format!("/api/applications/{social_id}/submit"),
            Some(&app.teacher),
            Some(json!({ "commitment_accepted": true })),
            StatusCode::OK,
        )
        .await?;
        if submitted["status"] != "submitted" {
            return Err(format!("社会科学类提交后状态不正确: {submitted}"));
        }
        expect_status(
            &app.app,
            Method::PUT,
            &format!("/api/applications/{social_id}/basic"),
            Some(&app.teacher),
            Some(basic_body("提交后修改")),
            StatusCode::CONFLICT,
        )
        .await?;
        expect_status(
            &app.app,
            Method::DELETE,
            &format!("/api/applications/{social_id}"),
            Some(&app.teacher),
            None,
            StatusCode::CONFLICT,
        )
        .await?;

        let natural = create_application(&app.app, &app.teacher, "natural").await?;
        let natural_id = natural["id"].as_str().ok_or("自科申报书缺少 id")?;
        expect_status(
            &app.app,
            Method::PUT,
            &format!("/api/applications/{natural_id}/ss-design"),
            Some(&app.teacher),
            Some(json!({ "topic_basis": "不应保存" })),
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .await?;
        save_basic(&app.app, &app.teacher, natural_id, "实训设备改造").await?;
        save_budget(&app.app, &app.teacher, natural_id, "ns_material", "2.00").await?;
        let submitted = expect_status(
            &app.app,
            Method::POST,
            &format!("/api/applications/{natural_id}/submit"),
            Some(&app.teacher),
            Some(json!({ "commitment_accepted": true })),
            StatusCode::OK,
        )
        .await?;
        if submitted["status"] != "submitted" || submitted["project_class"] != "natural" {
            return Err(format!("自然科学类提交结果不正确: {submitted}"));
        }

        let (status, mine) = call(
            &app.app,
            Method::GET,
            "/api/applications/mine",
            Some(&app.teacher),
            None,
        )
        .await?;
        let count = mine["items"].as_array().map(Vec::len).unwrap_or(0);
        if status != StatusCode::OK || count != 2 {
            return Err(format!("我的申报应有 2 项，实际 {status} {mine}"));
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "需要本地 PostgreSQL，使用独立测试库"]
async fn profile_changes_apply_to_new_applications_only() {
    with_test_app(|app| async move {
        let first = create_application(&app.app, &app.teacher, "social").await?;
        let (status, profile) = call(
            &app.app,
            Method::PATCH,
            "/api/profile",
            Some(&app.teacher),
            Some(json!({
                "professional_title": "教授",
                "phone": "13800001111",
                "education": "博士研究生",
                "birth_date": "1988-03"
            })),
        )
        .await?;
        if status != StatusCode::OK
            || profile["professional_title"] != "教授"
            || profile["birth_date"] != "1988-03"
        {
            return Err(format!("个人资料没有保存: {status} {profile}"));
        }
        let (status, _) = call(
            &app.app,
            Method::PATCH,
            "/api/profile",
            Some(&app.teacher),
            Some(json!({ "email": "flow-other@example.test" })),
        )
        .await?;
        if status != StatusCode::CONFLICT {
            return Err(format!("重复邮箱应返回 409，实际 {status}"));
        }
        let (status, _) = call(
            &app.app,
            Method::PATCH,
            "/api/profile",
            Some(&app.teacher),
            Some(json!({ "birth_date": "1988-13" })),
        )
        .await?;
        if status != StatusCode::UNPROCESSABLE_ENTITY {
            return Err(format!("无效出生年月应返回 422，实际 {status}"));
        }

        let second = create_application(&app.app, &app.teacher, "natural").await?;
        if second["professional_title"] != "教授"
            || second["mobile_phone"] != "13800001111"
            || second["education"] != "博士研究生"
        {
            return Err(format!("新建申报没有使用修改后的资料: {second}"));
        }
        let first_id = first["id"].as_str().ok_or("第一份申报缺少 id")?;
        let (status, saved) = call(
            &app.app,
            Method::GET,
            &format!("/api/applications/{first_id}"),
            Some(&app.teacher),
            None,
        )
        .await?;
        if status != StatusCode::OK || saved["professional_title"] != "副教授" {
            return Err(format!("已有申报书的负责人快照被个人资料修改影响: {saved}"));
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "需要本地 PostgreSQL，使用独立测试库"]
async fn teacher_downloads_form_pdf_and_others_cannot() {
    with_test_app(|app| async move {
        let created = create_application(&app.app, &app.teacher, "social").await?;
        let id = created["id"].as_str().ok_or("申报缺少 id")?;
        save_basic(&app.app, &app.teacher, id, "县域产业研究").await?;
        let path = format!("/api/applications/{id}/form.pdf");
        let (status, bytes) =
            call_bytes(&app.app, Method::GET, &path, Some(&app.teacher), None).await?;
        if status != StatusCode::OK || !bytes.starts_with(b"%PDF") {
            return Err(format!(
                "教师导出应返回 PDF，实际 {status}，长度 {}",
                bytes.len()
            ));
        }
        let (status, _) = call_bytes(&app.app, Method::GET, &path, Some(&app.other), None).await?;
        if status != StatusCode::FORBIDDEN {
            return Err(format!("其他教师导出应返回 403，实际 {status}"));
        }
        let (status, _) = call_bytes(&app.app, Method::GET, &path, None, None).await?;
        if status != StatusCode::UNAUTHORIZED {
            return Err(format!("未登录导出应返回 401，实际 {status}"));
        }
        let (status, office_bytes) =
            call_bytes(&app.app, Method::GET, &path, Some(&app.office), None).await?;
        if status != StatusCode::OK || !office_bytes.starts_with(b"%PDF") {
            return Err(format!("科研处导出应返回 PDF，实际 {status}"));
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "需要本地 PostgreSQL，使用独立测试库"]
async fn office_stats_match_counts_and_requested_funding() {
    with_test_app(|app| async move {
        let social = create_application(&app.app, &app.teacher, "social").await?;
        let social_id = social["id"].as_str().ok_or("社科申报缺少 id")?;
        save_basic(&app.app, &app.teacher, social_id, "县域产业研究").await?;
        save_budget(&app.app, &app.teacher, social_id, "ss_books", "1.20").await?;
        expect_status(
            &app.app,
            Method::POST,
            &format!("/api/applications/{social_id}/submit"),
            Some(&app.teacher),
            Some(json!({ "commitment_accepted": true })),
            StatusCode::OK,
        )
        .await?;
        let natural = create_application(&app.app, &app.other, "natural").await?;
        let natural_id = natural["id"].as_str().ok_or("自科申报缺少 id")?;
        save_budget(&app.app, &app.other, natural_id, "ns_material", "2.50").await?;

        let (status, _) = call(
            &app.app,
            Method::GET,
            "/api/stats/overview",
            Some(&app.teacher),
            None,
        )
        .await?;
        if status != StatusCode::FORBIDDEN {
            return Err(format!("教师查看统计应返回 403，实际 {status}"));
        }
        let (status, overview) = call(
            &app.app,
            Method::GET,
            "/api/stats/overview",
            Some(&app.office),
            None,
        )
        .await?;
        if status != StatusCode::OK {
            return Err(format!("科研处统计失败: {status} {overview}"));
        }
        if overview["total_count"] != 2 || overview["total_amount"] != "3.70" {
            return Err(format!("统计合计不正确: {overview}"));
        }
        let social_amount = bucket_amount(&overview, "by_class", "social")?;
        let natural_amount = bucket_amount(&overview, "by_class", "natural")?;
        let department_count = bucket_count(&overview, "by_department", "接口测试学院")?;
        if social_amount != "1.20" || natural_amount != "2.50" || department_count != 2 {
            return Err(format!("分类统计不正确: {overview}"));
        }
        let draft = bucket_count(&overview, "by_status", "草稿")?;
        let submitted = bucket_count(&overview, "by_status", "待系审核")?;
        if draft != 1 || submitted != 1 {
            return Err(format!("状态统计不正确: {overview}"));
        }

        let (status, bytes) = call_bytes(
            &app.app,
            Method::GET,
            "/api/stats/export",
            Some(&app.admin),
            None,
        )
        .await?;
        if status != StatusCode::OK || bytes.len() < 4 || &bytes[..2] != b"PK" {
            return Err(format!(
                "统计导出不是 xlsx，状态 {status}，长度 {}",
                bytes.len()
            ));
        }
        Ok(())
    })
    .await;
}

async fn create_application(app: &axum::Router, token: &str, class: &str) -> Result<Value, String> {
    let (status, body) = call(
        app,
        Method::POST,
        "/api/applications",
        Some(token),
        Some(json!({ "project_class": class })),
    )
    .await?;
    if status != StatusCode::CREATED {
        return Err(format!("新建 {class} 申报失败: {status} {body}"));
    }
    Ok(body)
}

async fn save_basic(app: &axum::Router, token: &str, id: &str, title: &str) -> Result<(), String> {
    expect_status(
        app,
        Method::PUT,
        &format!("/api/applications/{id}/basic"),
        Some(token),
        Some(basic_body(title)),
        StatusCode::OK,
    )
    .await?;
    Ok(())
}

async fn save_budget(
    app: &axum::Router,
    token: &str,
    id: &str,
    subject: &str,
    amount: &str,
) -> Result<(), String> {
    expect_status(
        app,
        Method::PUT,
        &format!("/api/applications/{id}/budget"),
        Some(token),
        Some(json!({ "lines": [{ "subject_code": subject, "amount": amount }] })),
        StatusCode::OK,
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[ignore = "需要本地 PostgreSQL，使用独立测试库"]
async fn teacher_uploads_downloads_and_deletes_attachment() {
    with_test_app(|app| async move {
        let created = create_application(&app.app, &app.teacher, "social").await?;
        let id = created["id"].as_str().ok_or("申报书缺少 id")?;
        if created["attachments"]
            .as_array()
            .is_none_or(|items| !items.is_empty())
        {
            return Err(format!("新建申报不应带附件: {created}"));
        }

        let (status, body) = upload_file(
            &app.app,
            &app.teacher,
            id,
            "supporting",
            "证明.png",
            TINY_PNG,
        )
        .await?;
        if status != StatusCode::CREATED
            || body["kind"] != "supporting"
            || body["file_name"] != "证明.png"
        {
            return Err(format!("上传附件失败: {status} {body}"));
        }
        let attachment_id = body["id"].as_str().ok_or("附件缺少 id")?;

        let detail = expect_status(
            &app.app,
            Method::GET,
            &format!("/api/applications/{id}"),
            Some(&app.teacher),
            None,
            StatusCode::OK,
        )
        .await?;
        if detail["attachments"].as_array().map(Vec::len) != Some(1) {
            return Err(format!("详情没有列出附件: {detail}"));
        }

        let (status, bytes) = download_file(&app.app, &app.teacher, id, attachment_id).await?;
        if status != StatusCode::OK || !bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            return Err(format!("下载内容不是 PNG: {status} {} 字节", bytes.len()));
        }
        let (status, _) = download_file(&app.app, &app.other, id, attachment_id).await?;
        if status != StatusCode::FORBIDDEN {
            return Err(format!("其他教师下载应返回 403，实际 {status}"));
        }

        let (status, body) = upload_file(
            &app.app,
            &app.teacher,
            id,
            "supporting",
            "说明.txt",
            b"plain text",
        )
        .await?;
        if status != StatusCode::UNPROCESSABLE_ENTITY {
            return Err(format!("文本文件应返回 422，实际 {status} {body}"));
        }

        expect_status(
            &app.app,
            Method::DELETE,
            &format!("/api/applications/{id}/attachments/{attachment_id}"),
            Some(&app.teacher),
            None,
            StatusCode::NO_CONTENT,
        )
        .await?;
        let detail = expect_status(
            &app.app,
            Method::GET,
            &format!("/api/applications/{id}"),
            Some(&app.teacher),
            None,
            StatusCode::OK,
        )
        .await?;
        if detail["attachments"].as_array().map(Vec::len) != Some(0) {
            return Err(format!("删除后仍有附件: {detail}"));
        }
        Ok(())
    })
    .await;
}

const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x44, 0x41, 0x54, 0x78, 0xDA, 0x63, 0xFC, 0xCF, 0xC0, 0x50,
    0x0F, 0x00, 0x04, 0x85, 0x01, 0x80, 0x84, 0xA9, 0x8C, 0x21, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

async fn upload_file(
    app: &axum::Router,
    token: &str,
    id: &str,
    kind: &str,
    file_name: &str,
    bytes: &[u8],
) -> Result<(StatusCode, Value), String> {
    let boundary = "researchapplyboundary";
    let mut payload = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"kind\"\r\n\r\n{kind}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
    .into_bytes();
    payload.extend_from_slice(bytes);
    payload.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    send_raw(
        app,
        Method::POST,
        &format!("/api/applications/{id}/attachments"),
        token,
        &format!("multipart/form-data; boundary={boundary}"),
        payload,
    )
    .await
}

async fn download_file(
    app: &axum::Router,
    token: &str,
    id: &str,
    attachment_id: &str,
) -> Result<(StatusCode, Vec<u8>), String> {
    let request = axum::http::Request::builder()
        .method(Method::GET)
        .uri(format!(
            "/api/applications/{id}/attachments/{attachment_id}"
        ))
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .map_err(|error| error.to_string())?;
    let response = app
        .clone()
        .oneshot(request)
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .map_err(|error| error.to_string())?
        .to_bytes()
        .to_vec();
    Ok((status, bytes))
}

async fn send_raw(
    app: &axum::Router,
    method: Method,
    path: &str,
    token: &str,
    content_type: &str,
    payload: Vec<u8>,
) -> Result<(StatusCode, Value), String> {
    let request = axum::http::Request::builder()
        .method(method)
        .uri(path)
        .header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"))
        .header(axum::http::header::CONTENT_TYPE, content_type)
        .body(axum::body::Body::from(payload))
        .map_err(|error| error.to_string())?;
    let response = app
        .clone()
        .oneshot(request)
        .await
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .map_err(|error| error.to_string())?
        .to_bytes();
    let value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    Ok((status, value))
}

async fn expect_status(
    app: &axum::Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
    expected: StatusCode,
) -> Result<Value, String> {
    let label = format!("{method} {path}");
    let (status, value) = call(app, method, path, token, body).await?;
    if status != expected {
        return Err(format!("{label} 期望 {expected}，实际 {status} {value}"));
    }
    Ok(value)
}

fn basic_body(title: &str) -> Value {
    json!({
        "title": title,
        "keywords": "产业;县域",
        "professional_title": "副教授",
        "education": "研究生",
        "degree": "硕士"
    })
}

fn bucket_amount(overview: &Value, group: &str, code_or_label: &str) -> Result<String, String> {
    bucket(overview, group, code_or_label)?
        .get("amount")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("{group} 中没有 {code_or_label} 的金额"))
}

fn bucket_count(overview: &Value, group: &str, code_or_label: &str) -> Result<i64, String> {
    bucket(overview, group, code_or_label)?
        .get("count")
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{group} 中没有 {code_or_label} 的数量"))
}

fn bucket<'a>(overview: &'a Value, group: &str, code_or_label: &str) -> Result<&'a Value, String> {
    overview[group]
        .as_array()
        .ok_or_else(|| format!("{group} 不是数组"))?
        .iter()
        .find(|item| item["code"] == code_or_label || item["label"] == code_or_label)
        .ok_or_else(|| format!("{group} 中没有 {code_or_label}"))
}
