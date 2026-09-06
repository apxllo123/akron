use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::manifest::{GameManifest, ProtectionSignals};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameProfile {
    pub executables: Vec<ExecutableProfile>,
    pub graphics: GraphicsRequirements,
    pub windows_apis: Vec<WindowsApiRequirement>,
    pub runtimes: Vec<RuntimeRequirement>,
    pub protections: ProtectionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutableProfile {
    pub path: String,
    pub architecture: Option<String>,
    pub format: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphicsRequirements {
    pub direct3d9: bool,
    pub direct3d10: bool,
    pub direct3d11: bool,
    pub direct3d12: bool,
    pub dxgi: bool,
    pub vulkan: bool,
    pub opengl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowsApiRequirement {
    pub family: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeRequirement {
    pub name: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtectionSummary {
    pub packers_or_protectors: Vec<String>,
    pub anti_cheats: Vec<String>,
}

pub fn profile_game(manifest: &GameManifest) -> Result<GameProfile> {
    let mut profile = GameProfile {
        executables: manifest
            .executables
            .iter()
            .map(|exe| ExecutableProfile {
                path: exe.path.to_string_lossy().into_owned(),
                architecture: exe.architecture.clone(),
                format: exe.format.clone(),
            })
            .collect(),
        graphics: GraphicsRequirements::default(),
        windows_apis: Vec::new(),
        runtimes: Vec::new(),
        protections: ProtectionSummary::default(),
    };

    let mut api_evidence = std::collections::BTreeMap::<String, Vec<String>>::new();
    let mut runtime_evidence = std::collections::BTreeMap::<String, Vec<String>>::new();

    for exe in &manifest.executables {
        merge_protections(&mut profile.protections, &exe.protection);
        let absolute = manifest.root.join(&exe.path);
        scan_binary(
            &absolute,
            &mut profile.graphics,
            &mut api_evidence,
            &mut runtime_evidence,
        )
        .with_context(|| format!("failed to profile {}", absolute.display()))?;
    }

    profile.windows_apis = api_evidence
        .into_iter()
        .map(|(family, mut evidence)| {
            evidence.sort();
            WindowsApiRequirement { family, evidence }
        })
        .collect();

    profile.runtimes = runtime_evidence
        .into_iter()
        .map(|(name, mut evidence)| {
            evidence.sort();
            RuntimeRequirement { name, evidence }
        })
        .collect();

    Ok(profile)
}

fn merge_protections(summary: &mut ProtectionSummary, signals: &ProtectionSignals) {
    for value in &signals.packers_or_protectors {
        if !summary.packers_or_protectors.contains(value) {
            summary.packers_or_protectors.push(value.clone());
        }
    }
    for value in &signals.anti_cheats {
        if !summary.anti_cheats.contains(value) {
            summary.anti_cheats.push(value.clone());
        }
    }
    summary.packers_or_protectors.sort();
    summary.anti_cheats.sort();
}

fn scan_binary(
    path: &Path,
    graphics: &mut GraphicsRequirements,
    api_evidence: &mut std::collections::BTreeMap<String, Vec<String>>,
    runtime_evidence: &mut std::collections::BTreeMap<String, Vec<String>>,
) -> Result<()> {
    let mut file = File::open(path)?;
    let mut buffer = [0_u8; 1024 * 1024];
    let mut tail = Vec::<u8>::new();

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }

        let mut chunk = Vec::with_capacity(tail.len() + read);
        chunk.extend_from_slice(&tail);
        chunk.extend_from_slice(&buffer[..read]);

        detect_graphics(&chunk, graphics);
        detect_windows_apis(&chunk, path, api_evidence);
        detect_runtimes(&chunk, path, runtime_evidence);

        const MAX_MARKER_LEN: usize = 32;
        let keep = MAX_MARKER_LEN.saturating_sub(1).min(chunk.len());
        tail.clear();
        tail.extend_from_slice(&chunk[chunk.len() - keep..]);
    }

    Ok(())
}

fn detect_graphics(data: &[u8], graphics: &mut GraphicsRequirements) {
    graphics.direct3d9 |= contains_any(data, &[b"d3d9.dll", b"d3d9_43.dll"]);
    graphics.direct3d10 |= contains_any(data, &[b"d3d10.dll", b"d3d10_1.dll"]);
    graphics.direct3d11 |= contains_any(data, &[b"d3d11.dll"]);
    graphics.direct3d12 |= contains_any(data, &[b"d3d12.dll"]);
    graphics.dxgi |= contains_any(data, &[b"dxgi.dll"]);
    graphics.vulkan |= contains_any(data, &[b"vulkan-1.dll", b"vulkan"]);
    graphics.opengl |= contains_any(data, &[b"opengl32.dll"]);
}

fn detect_windows_apis(
    data: &[u8],
    path: &Path,
    evidence: &mut std::collections::BTreeMap<String, Vec<String>>,
) {
    const FAMILIES: &[(&[u8], &str)] = &[
        (b"kernel32.dll", "process-threading/filesystem"),
        (b"kernelbase.dll", "process-threading/filesystem"),
        (b"user32.dll", "windowing/input"),
        (b"gdi32.dll", "windowing/2d-graphics"),
        (b"advapi32.dll", "registry/security/services"),
        (b"shell32.dll", "shell/integration"),
        (b"ole32.dll", "com"),
        (b"ws2_32.dll", "networking"),
        (b"winhttp.dll", "http-networking"),
        (b"winmm.dll", "legacy-audio/timing"),
        (b"xaudio2", "audio"),
        (b"xinput", "controller-input"),
        (b"hid.dll", "raw-input"),
    ];

    for &(marker, family) in FAMILIES {
        if contains_case_insensitive(data, marker) {
            evidence
                .entry(family.to_owned())
                .or_default()
                .push(path.display().to_string());
        }
    }
}

fn detect_runtimes(
    data: &[u8],
    path: &Path,
    evidence: &mut std::collections::BTreeMap<String, Vec<String>>,
) {
    const RUNTIMES: &[(&[u8], &str)] = &[
        (b"vcruntime140.dll", "Microsoft Visual C++ 2015-2022"),
        (b"msvcp140.dll", "Microsoft Visual C++ 2015-2022"),
        (b"ucrtbase.dll", "Universal C Runtime"),
        (b"msvcp120.dll", "Microsoft Visual C++ 2013"),
        (b"msvcr120.dll", "Microsoft Visual C++ 2013"),
        (b"msvcp110.dll", "Microsoft Visual C++ 2012"),
        (b"msvcr110.dll", "Microsoft Visual C++ 2012"),
        (b"mscoree.dll", ".NET Framework CLR"),
    ];

    for &(marker, runtime) in RUNTIMES {
        if contains_case_insensitive(data, marker) {
            evidence
                .entry(runtime.to_owned())
                .or_default()
                .push(path.display().to_string());
        }
    }
}

fn contains_any(data: &[u8], needles: &[&[u8]]) -> bool {
    needles
        .iter()
        .any(|needle| contains_case_insensitive(data, needle))
}

fn contains_case_insensitive(data: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    data.windows(needle.len()).any(|window| {
        window
            .iter()
            .zip(needle)
            .all(|(&a, &b)| a.eq_ignore_ascii_case(&b))
    })
}

#[cfg(test)]
mod tests {
    use super::{GameProfile, GraphicsRequirements, profile_game};
    use crate::manifest::{ExecutableRecord, GameManifest, ProtectionSignals};
    use std::path::PathBuf;

    #[test]
    fn profiles_graphics_and_runtime_markers() {
        let root = std::env::temp_dir().join("akron-profile-fixture");
        std::fs::create_dir_all(&root).expect("create fixture dir");
        let exe = root.join("game.exe");
        std::fs::write(
            &exe,
            b"d3d11.dll dxgi.dll vcruntime140.dll user32.dll ws2_32.dll",
        )
        .expect("write fixture");

        let manifest = GameManifest {
            root: root.clone(),
            files: Vec::new(),
            executables: vec![ExecutableRecord {
                path: PathBuf::from("game.exe"),
                format: "PE".to_owned(),
                architecture: Some("x86_64".to_owned()),
                protection: ProtectionSignals::default(),
            }],
        };

        let profile: GameProfile = profile_game(&manifest).expect("profile");
        assert_eq!(
            profile.graphics,
            GraphicsRequirements {
                direct3d11: true,
                dxgi: true,
                ..GraphicsRequirements::default()
            }
        );
        assert!(
            profile
                .windows_apis
                .iter()
                .any(|v| v.family == "windowing/input")
        );
        assert!(
            profile
                .windows_apis
                .iter()
                .any(|v| v.family == "networking")
        );
        assert!(
            profile
                .runtimes
                .iter()
                .any(|v| v.name == "Microsoft Visual C++ 2015-2022")
        );

        std::fs::remove_file(exe).expect("remove fixture");
        std::fs::remove_dir(root).expect("remove fixture dir");
    }
}
