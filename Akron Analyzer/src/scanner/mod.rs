use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use walkdir::WalkDir;

use crate::manifest::{ExecutableRecord, FileRecord, GameManifest};

pub fn analyze_game(root: &Path) -> Result<GameManifest> {
    let root = root
        .canonicalize()
        .with_context(|| format!("failed to resolve game path: {}", root.display()))?;

    if !root.is_dir() {
        anyhow::bail!("input is not a directory: {}", root.display());
    }

    let mut files = Vec::new();
    let mut executables = Vec::new();

    for entry in WalkDir::new(&root).follow_links(false) {
        let entry = entry.with_context(|| "failed while walking game directory")?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let relative = path.strip_prefix(&root).unwrap_or(path).to_path_buf();
        let metadata = entry
            .metadata()
            .with_context(|| format!("failed to stat {}", path.display()))?;
        let sha256 = sha256_file(path)?;
        let extension = path
            .extension()
            .and_then(|v| v.to_str())
            .map(str::to_ascii_lowercase);

        files.push(FileRecord {
            path: relative.clone(),
            size: metadata.len(),
            sha256,
            extension: extension.clone(),
        });

        if matches!(extension.as_deref(), Some("exe" | "dll" | "sys")) {
            executables.push(ExecutableRecord {
                path: relative,
                format: detect_binary_format(path)?,
                architecture: detect_pe_architecture(path)?,
            });
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    executables.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(GameManifest {
        root,
        files,
        executables,
    })
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn detect_binary_format(path: &Path) -> Result<String> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut header = [0_u8; 4];
    let read = file.read(&mut header)?;

    Ok(if read >= 2 && &header[..2] == b"MZ" {
        "PE".to_owned()
    } else {
        "unknown".to_owned()
    })
}

fn detect_pe_architecture(path: &Path) -> Result<Option<String>> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut dos_header = [0_u8; 0x40];
    let read = file.read(&mut dos_header)?;
    if read < dos_header.len() || &dos_header[..2] != b"MZ" {
        return Ok(None);
    }

    let pe_offset = u32::from_le_bytes(dos_header[0x3c..0x40].try_into().unwrap()) as u64;
    file.seek(SeekFrom::Start(pe_offset))?;

    let mut pe_header = [0_u8; 6];
    let read = file.read(&mut pe_header)?;
    if read < pe_header.len() || &pe_header[..4] != b"PE\0\0" {
        return Ok(None);
    }

    let machine = u16::from_le_bytes(pe_header[4..6].try_into().unwrap());
    let arch = match machine {
        0x014c => "x86",
        0x8664 => "x86_64",
        0xAA64 => "arm64",
        0x01c4 => "arm32",
        _ => return Ok(Some(format!("machine-0x{machine:04x}"))),
    };

    Ok(Some(arch.to_owned()))
}
