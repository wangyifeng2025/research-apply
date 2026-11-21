//! 附件文件存储。业务层只保存 `storage_key`，不关心文件放在哪里。
//!
//! 当前由 `LocalDirectory` 写入本地目录。换成对象存储时替换这三个方法，
//! 上传、下载和删除仍使用同一个存储键。

use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::error::AppError;

pub const MAX_BYTES: u64 = 20 * 1024 * 1024;
pub const REQUEST_LIMIT: usize = (MAX_BYTES as usize) + 64 * 1024;

#[derive(Clone, Debug)]
pub struct LocalDirectory {
    root: PathBuf,
}

impl LocalDirectory {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn location(&self, key: &str) -> Result<PathBuf, AppError> {
        if !valid_storage_key(key) {
            return Err(AppError::NotFound("附件不存在".to_owned()));
        }
        Ok(self.root.join(key))
    }

    pub async fn save(&self, key: &str, bytes: &[u8]) -> Result<(), AppError> {
        let path = self.location(key)?;
        if let Err(error) = tokio::fs::create_dir_all(&self.root).await {
            tracing::error!(error = %error, "创建附件目录失败");
            return Err(AppError::Unavailable(
                "附件保存失败，请联系管理员检查附件目录".to_owned(),
            ));
        }
        let partial = self.root.join(format!(".{key}.partial"));
        if let Err(error) = tokio::fs::write(&partial, bytes).await {
            tracing::error!(error = %error, "写入附件失败");
            return Err(AppError::Unavailable(
                "附件保存失败，请联系管理员检查附件目录".to_owned(),
            ));
        }
        if let Err(error) = tokio::fs::rename(&partial, &path).await {
            let _ = tokio::fs::remove_file(&partial).await;
            tracing::error!(error = %error, "保存附件失败");
            return Err(AppError::Unavailable(
                "附件保存失败，请联系管理员检查附件目录".to_owned(),
            ));
        }
        Ok(())
    }

    pub async fn load(&self, key: &str) -> Result<Vec<u8>, AppError> {
        let path = self.location(key)?;
        match tokio::fs::read(&path).await {
            Ok(bytes) => Ok(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Err(AppError::NotFound("附件不存在".to_owned()))
            }
            Err(error) => Err(error.into()),
        }
    }

    pub async fn remove(&self, key: &str) -> Result<(), AppError> {
        let path = self.location(key)?;
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

pub fn valid_storage_key(key: &str) -> bool {
    let Some((id, extension)) = key.split_once('.') else {
        return false;
    };
    Uuid::parse_str(id).is_ok()
        && !extension.contains('.')
        && matches!(extension, "pdf" | "jpg" | "png")
}

pub fn exceeds_limit(size: u64) -> bool {
    size > MAX_BYTES
}

/// 根据文件头判断类型，不信任浏览器申报的 Content-Type。
///
/// 允许文件开头有 UTF-8 BOM 或少量空白。部分扫描件和办公软件导出的 PDF 会这样。
pub fn sniff(bytes: &[u8]) -> Option<DetectedFile> {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    let offset = bytes
        .iter()
        .take(1024)
        .position(|byte| !matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
        .unwrap_or(0);
    let bytes = bytes.get(offset..).unwrap_or(bytes);
    if bytes.starts_with(b"%PDF") {
        Some(DetectedFile::Pdf)
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some(DetectedFile::Jpeg)
    } else if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some(DetectedFile::Png)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetectedFile {
    Pdf,
    Jpeg,
    Png,
}

impl DetectedFile {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Jpeg => "jpg",
            Self::Png => "png",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
        }
    }

    pub fn matches_name(self, file_name: &str) -> bool {
        let Some((_, extension)) = file_name.rsplit_once('.') else {
            return true;
        };
        let extension = extension.to_ascii_lowercase();
        match self {
            Self::Pdf => extension == "pdf",
            Self::Jpeg => extension == "jpg" || extension == "jpeg",
            Self::Png => extension == "png",
        }
    }
}

pub fn safe_file_name(raw: &str) -> String {
    let base = Path::new(raw)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("附件");
    let cleaned: String = base
        .chars()
        .filter(|ch| !ch.is_control() && *ch != '"' && *ch != '\\')
        .take(255)
        .collect();
    if cleaned.is_empty() {
        "附件".to_owned()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::{DetectedFile, MAX_BYTES, exceeds_limit, safe_file_name, sniff, valid_storage_key};

    #[test]
    fn sniff_accepts_pdf_jpeg_and_png_headers() {
        assert_eq!(sniff(b"%PDF-1.7"), Some(DetectedFile::Pdf));
        assert_eq!(sniff(&[0xFF, 0xD8, 0xFF, 0xE0]), Some(DetectedFile::Jpeg));
        assert_eq!(
            sniff(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00]),
            Some(DetectedFile::Png)
        );
        assert_eq!(sniff(b"not a file"), None);
        assert_eq!(sniff(b"\xEF\xBB\xBF%PDF-1.4"), Some(DetectedFile::Pdf));
        assert_eq!(sniff(b"\n%PDF-1.7"), Some(DetectedFile::Pdf));
    }

    #[test]
    fn storage_key_rejects_paths() {
        assert!(valid_storage_key(
            "11111111-1111-1111-1111-111111111111.pdf"
        ));
        assert!(!valid_storage_key("../secret.pdf"));
        assert!(!valid_storage_key(
            "11111111-1111-1111-1111-111111111111.txt"
        ));
        assert!(!valid_storage_key("notes.pdf"));
    }

    #[test]
    fn twenty_megabytes_is_the_limit() {
        assert!(!exceeds_limit(MAX_BYTES));
        assert!(exceeds_limit(MAX_BYTES + 1));
    }

    #[test]
    fn file_name_drops_directories_and_quotes() {
        assert_eq!(safe_file_name("../../签字件\".png"), "签字件.png");
        assert_eq!(safe_file_name(""), "附件");
    }

    #[test]
    fn extension_must_match_detected_type() {
        assert!(DetectedFile::Jpeg.matches_name("照片.JPEG"));
        assert!(!DetectedFile::Png.matches_name("照片.jpg"));
        assert!(DetectedFile::Pdf.matches_name("无扩展名"));
    }
}
