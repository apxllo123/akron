use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use crate::manifest::{GameManifest, ProtectionSignals};
use crate::pe::{PeBinaryAnalysis, analyze_pe};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameProfile {
    pub executables: Vec<ExecutableProfile>,
    pub pe_binaries: Vec<PeBinaryProfile>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeBinaryProfile {
    pub path: String,
    pub architecture: String,
    pub kind: String,
    pub imports: usize,
    pub libraries: Vec<String>,
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
        pe_binaries: Vec::new(),
        graphics: GraphicsRequirements::default(),
        windows_apis: Vec::new(),
        runtimes: Vec::new(),
        protections: ProtectionSummary::default(),
    };

    let mut api_evidence = BTreeMap::<String, Vec<String>>::new();
    let mut runtime_evidence = BTreeMap::<String, Vec<String>>::new();

    for exe in &manifest.executables {
        merge_protections(&mut profile.protections, &exe.protection);
        if !exe.format.eq_ignore_ascii_case("PE") {
            continue;
        }

        let absolute = manifest.root.join(&exe.path);
        let analysis = analyze_pe(&absolute)
            .with_context(|| format!("failed to profile PE {}", absolute.display()))?;

        apply_pe_requirements(
            &analysis,
            &exe.path,
            &mut profile.graphics,
            &mut api_evidence,
            &mut runtime_evidence,
        );

        profile.pe_binaries.push(PeBinaryProfile {
            path: exe.path.to_string_lossy().into_owned(),
            architecture: if analysis.is_64 { "x86_64" } else { "x86" }.to_owned(),
            kind: if analysis.is_library { "DLL" } else { "EXE" }.to_owned(),
            imports: analysis.imports.len(),
            libraries: analysis.libraries.clone(),
        });
    }

    profile.pe_binaries.sort_by(|a, b| a.path.cmp(&b.path));
    profile.windows_apis = into_api_requirements(api_evidence);
    profile.runtimes = into_runtime_requirements(runtime_evidence);
    Ok(profile)
}

fn apply_pe_requirements(
    analysis: &PeBinaryAnalysis,
    path: &Path,
    graphics: &mut GraphicsRequirements,
    api_evidence: &mut BTreeMap<String, Vec<String>>,
    runtime_evidence: &mut BTreeMap<String, Vec<String>>,
) {
    for library in &analysis.libraries {
        let library = library.to_ascii_lowercase();
        let source = path.display().to_string();
        match library.as_str() {
            "d3d9.dll" | "d3d9_43.dll" => {
                graphics.direct3d9 = true;
                add_evidence(api_evidence, "graphics/direct3d9", source.clone());
            }
            "d3d10.dll" | "d3d10_1.dll" => {
                graphics.direct3d10 = true;
                add_evidence(api_evidence, "graphics/direct3d10", source.clone());
            }
            "d3d11.dll" => {
                graphics.direct3d11 = true;
                add_evidence(api_evidence, "graphics/direct3d11", source.clone());
            }
            "d3d12.dll" => {
                graphics.direct3d12 = true;
                add_evidence(api_evidence, "graphics/direct3d12", source.clone());
            }
            "dxgi.dll" => {
                graphics.dxgi = true;
                add_evidence(api_evidence, "graphics/dxgi", source.clone());
            }
            "vulkan-1.dll" => {
                graphics.vulkan = true;
                add_evidence(api_evidence, "graphics/vulkan", source.clone());
            }
            "opengl32.dll" => {
                graphics.opengl = true;
                add_evidence(api_evidence, "graphics/opengl", source.clone());
            }
            "kernel32.dll" | "kernelbase.dll" => {
                add_evidence(api_evidence, "process-threading/filesystem", source.clone());
            }
            "user32.dll" => add_evidence(api_evidence, "windowing/input", source.clone()),
            "gdi32.dll" => add_evidence(api_evidence, "windowing/2d-graphics", source.clone()),
            "advapi32.dll" => add_evidence(api_evidence, "registry/security/services", source.clone()),
            "shell32.dll" => add_evidence(api_evidence, "shell/integration", source.clone()),
            "ole32.dll" => add_evidence(api_evidence, "com", source.clone()),
            "ws2_32.dll" => add_evidence(api_evidence, "networking", source.clone()),
            "winhttp.dll" => add_evidence(api_evidence, "http-networking", source.clone()),
            "winmm.dll" => add_evidence(api_evidence, "legacy-audio/timing", source.clone()),
            "xaudio2_9.dll" | "xaudio2_8.dll" | "xaudio2_7.dll" => {
                add_evidence(api_evidence, "audio", source.clone());
            }
            "xinput1_4.dll" | "xinput1_3.dll" | "xinput9_1_0.dll" => {
                add_evidence(api_evidence, "controller-input", source.clone());
            }
            "hid.dll" => add_evidence(api_evidence, "raw-input", source.clone()),
            "vcruntime140.dll" | "vcruntime140_1.dll" | "msvcp140.dll" => {
                add_evidence(runtime_evidence, "Microsoft Visual C++ 2015-2022", source.clone());
            }
            "ucrtbase.dll" => add_evidence(runtime_evidence, "Universal C Runtime", source.clone()),
            "msvcp120.dll" | "msvcr120.dll" => {
                add_evidence(runtime_evidence, "Microsoft Visual C++ 2013", source.clone());
            }
            "msvcp110.dll" | "msvcr110.dll" => {
                add_evidence(runtime_evidence, "Microsoft Visual C++ 2012", source.clone());
            }
            "mscoree.dll" => add_evidence(runtime_evidence, ".NET Framework CLR", source.clone()),
            _ => {}
        }
    }

    for import in &analysis.imports {
        if !is_graphics_library(&import.library) {
            continue;
        }
        let source = match &import.name {
            Some(name) => format!("{} -> {}!{}", path.display(), import.library, name),
            None => format!(
                "{} -> {}!#{}",
                path.display(),
                import.library,
                import.ordinal.unwrap_or_default()
            ),
        };
        add_evidence(api_evidence, graphics_family(&import.library), source);
    }
}

fn is_graphics_library(library: &str) -> bool {
    matches!(
        library,
        "d3d9.dll"
            | "d3d9_43.dll"
            | "d3d10.dll"
            | "d3d10_1.dll"
            | "d3d11.dll"
            | "d3d12.dll"
            | "dxgi.dll"
            | "vulkan-1.dll"
            | "opengl32.dll"
    )
}

fn graphics_family(library: &str) -> &str {
    match library {
        "d3d9.dll" | "d3d9_43.dll" => "graphics/direct3d9",
        "d3d10.dll" | "d3d10_1.dll" => "graphics/direct3d10",
        "d3d11.dll" => "graphics/direct3d11",
        "d3d12.dll" => "graphics/direct3d12",
        "dxgi.dll" => "graphics/dxgi",
        "vulkan-1.dll" => "graphics/vulkan",
        "opengl32.dll" => "graphics/opengl",
        _ => "graphics/unknown",
    }
}

fn add_evidence(map: &mut BTreeMap<String, Vec<String>>, key: &str, value: String) {
    let values = map.entry(key.to_owned()).or_default();
    if !values.contains(&value) {
        values.push(value);
    }
}

fn into_api_requirements(map: BTreeMap<String, Vec<String>>) -> Vec<WindowsApiRequirement> {
    map.into_iter()
        .map(|(family, mut evidence)| {
            evidence.sort();
            WindowsApiRequirement { family, evidence }
        })
        .collect()
}

fn into_runtime_requirements(map: BTreeMap<String, Vec<String>>) -> Vec<RuntimeRequirement> {
    map.into_iter()
        .map(|(name, mut evidence)| {
            evidence.sort();
            RuntimeRequirement { name, evidence }
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::{GraphicsRequirements, apply_pe_requirements, profile_game};
    use crate::manifest::{ExecutableRecord, GameManifest, ProtectionSignals};
    use crate::pe::{PeBinaryAnalysis, PeImport};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn profiles_real_pe_import_requirements() {
        let analysis = PeBinaryAnalysis {
            is_64: true,
            is_library: false,
            entry_point_rva: 0x1000,
            image_base: 0x140000000,
            libraries: vec![
                "d3d11.dll".to_owned(),
                "dxgi.dll".to_owned(),
                "user32.dll".to_owned(),
                "ws2_32.dll".to_owned(),
                "vcruntime140.dll".to_owned(),
            ],
            imports: vec![PeImport {
                library: "d3d11.dll".to_owned(),
                name: Some("D3D11CreateDevice".to_owned()),
                ordinal: None,
                rva: 0x2000,
            }],
            sections: Vec::new(),
        };

        let mut graphics = GraphicsRequirements::default();
        let mut apis = BTreeMap::new();
        let mut runtimes = BTreeMap::new();
        apply_pe_requirements(
            &analysis,
            std::path::Path::new("game.exe"),
            &mut graphics,
            &mut apis,
            &mut runtimes,
        );

        assert!(graphics.direct3d11);
        assert!(graphics.dxgi);
        assert!(apis.contains_key("windowing/input"));
        assert!(apis.contains_key("networking"));
        assert!(runtimes.contains_key("Microsoft Visual C++ 2015-2022"));
        assert!(apis["graphics/direct3d11"][0].contains("D3D11CreateDevice"));
    }

    #[test]
    fn raw_marker_strings_are_not_authoritative_requirements() {
        let root = std::env::temp_dir().join("akron-profile-marker-fixture");
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

        let result = profile_game(&manifest);
        assert!(result.is_err(), "invalid PE must not be treated as a real import table");

        std::fs::remove_file(exe).expect("remove fixture");
        std::fs::remove_dir(root).expect("remove fixture dir");
    }
}
