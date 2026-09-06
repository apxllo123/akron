use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use walkdir::WalkDir;

use crate::manifest::{ExecutableRecord, FileRecord, GameManifest, ProtectionSignals};

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
                path: relative.clone(),
                format: detect_binary_format(path)?,
                architecture: detect_pe_architecture(path)?,
                protection: detect_protection_signals(path, &relative)?,
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

/// Detects well-known executable protection markers as non-authoritative signals.
///
/// This deliberately does not attempt to disable, bypass, or modify protections.
/// The results are intended for adaptation planning and diagnostics only.
fn detect_protection_signals(path: &Path, relative_path: &Path) -> Result<ProtectionSignals> {
    let mut file =
        File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    let lower_name = relative_path.to_string_lossy().to_ascii_lowercase();
    let lower_bytes = String::from_utf8_lossy(&data).to_ascii_lowercase();

    let mut protection = ProtectionSignals::default();

    for (marker, label) in [
        ("upx0", "UPX"),
        ("upx1", "UPX"),
        ("upx2", "UPX"),
        ("aspack", "ASPack"),
        ("mpress", "MPRESS"),
        ("themida", "Themida"),
        ("vmprotect", "VMProtect"),
        ("enigma", "Enigma Protector"),
    ] {
        if lower_bytes.contains(marker) && !protection.packers_or_protectors.iter().any(|v| v == label) {
            protection.packers_or_protectors.push(label.to_owned());
        }
    }

    for (marker, label) in [
        ("easyanticheat", "Easy Anti-Cheat"),
        ("easyanticheat_eos", "Easy Anti-Cheat EOS"),
        ("battleye", "BattlEye"),
        ("bedaisy", "BattlEye"),
        ("vgk", "Riot Vanguard"),
        ("vanguard", "Riot Vanguard"),
        ("gameguard", "nProtect GameGuard"),
        ("xigncode", "XIGNCODE3"),
    ] {
        if (lower_name.contains(marker) || lower_bytes.contains(marker))
            && !protection.anti_cheats.iter().any(|v| v == label)
        {
            protection.anti_cheats.push(label.to_owned());
        }
    }

    protection.packers_or_protectors.sort();
    protection.anti_cheats.sort();

    Ok(protection)
}

#[cfg(test)]
mod tests {
    use super::detect_protection_signals;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detects_known_protection_markers_without_modifying_input() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("akron-protection-{unique}.exe"));
        let data = b"This contains UPX0, VMProtect, and EasyAntiCheat markers.";
        fs::write(&path, data).expect("write fixture");

        let before = fs::read(&path).expect("read fixture");
        let signals = detect_protection_signals(&path, path.file_name().unwrap().as_ref())
            .expect("detect signals");
        let after = fs::read(&path).expect("read fixture after analysis");

        assert_eq!(before, after);
        assert!(signals.packers_or_protectors.contains(&"UPX".to_owned()));
        assert!(signals.packers_or_protectors.contains(&"VMProtect".to_owned()));
        assert!(signals.anti_cheats.contains(&"Easy Anti-Cheat".to_owned()));

        fs::remove_file(path).expect("remove fixture");
    }

    #[test]
    fn ignores_unrelated_data() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("akron-protection-clean-{unique}.exe"));
        fs::write(&path, b"ordinary executable data").expect("write fixture");

        let signals = detect_protection_signals(&path, path.file_name().unwrap().as_ref())
            .expect("detect signals");
        assert!(signals.packers_or_protectors.is_empty());
        assert!(signals.anti_cheats.is_empty());

        fs::remove_file(path).expect("remove fixture");
    }
}
