use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct GameManifest {
    pub root: PathBuf,
    pub files: Vec<FileRecord>,
    pub executables: Vec<ExecutableRecord>,
}

#[derive(Debug, Serialize)]
pub struct FileRecord {
    pub path: PathBuf,
    pub size: u64,
    pub sha256: String,
    pub extension: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutableRecord {
    pub path: PathBuf,
    pub format: String,
    pub architecture: Option<String>,
}
