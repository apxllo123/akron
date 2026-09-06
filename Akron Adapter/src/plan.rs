use akron_analyzer::GameProfile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdaptationPlan {
    pub steps: Vec<AdaptationStep>,
    pub required_modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdaptationStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub module: String,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Planned,
    Ready,
    Blocked,
}

pub fn build_plan(profile: &GameProfile) -> AdaptationPlan {
    let mut steps = Vec::new();
    let mut modules = Vec::new();

    add_step(
        &mut steps,
        &mut modules,
        "analyze",
        "Analyze game",
        "Review the detected PE binaries, graphics, API, runtime, dependency, and protection requirements.",
        "core.analyzer",
        StepStatus::Ready,
    );

    if profile.graphics.direct3d9 {
        add_translation_step(
            &mut steps,
            &mut modules,
            "graphics-d3d9",
            "Prepare Direct3D 9 → Metal",
            "A D3D9 conversion executor is not registered yet, so Akron will not claim this step is executable.",
            "graphics.d3d9",
        );
    }
    if profile.graphics.direct3d10 {
        add_translation_step(
            &mut steps,
            &mut modules,
            "graphics-d3d10",
            "Prepare Direct3D 10 → Metal",
            "A D3D10 conversion executor is not registered yet, so Akron will not claim this step is executable.",
            "graphics.d3d10",
        );
    }
    if profile.graphics.direct3d11 {
        add_translation_step(
            &mut steps,
            &mut modules,
            "graphics-d3d11",
            "Prepare Direct3D 11 → Metal",
            "A D3D11 conversion executor is not registered yet, so Akron will not claim this step is executable.",
            "graphics.d3d11",
        );
    }
    if profile.graphics.direct3d12 {
        add_translation_step(
            &mut steps,
            &mut modules,
            "graphics-d3d12",
            "Prepare Direct3D 12 → Metal",
            "A D3D12 conversion executor is not registered yet, so Akron will not claim this step is executable.",
            "graphics.d3d12",
        );
    }
    if profile.graphics.dxgi {
        add_translation_step(
            &mut steps,
            &mut modules,
            "graphics-dxgi",
            "Map DXGI requirements",
            "A DXGI adaptation executor is not registered yet; this requirement remains blocked rather than being reported as completed.",
            "graphics.dxgi",
        );
    }
    if profile.graphics.vulkan {
        add_translation_step(
            &mut steps,
            &mut modules,
            "graphics-vulkan",
            "Prepare Vulkan path",
            "A Vulkan adaptation executor is not registered yet; Akron only records the detected requirement.",
            "graphics.vulkan",
        );
    }
    if profile.graphics.opengl {
        add_translation_step(
            &mut steps,
            &mut modules,
            "graphics-opengl",
            "Prepare OpenGL path",
            "An OpenGL adaptation executor is not registered yet; Akron only records the detected requirement.",
            "graphics.opengl",
        );
    }

    for api in &profile.windows_apis {
        let id = format!("api-{}", slug(&api.family));
        add_translation_step(
            &mut steps,
            &mut modules,
            &id,
            &format!("Map {} API family", api.family),
            "A native implementation for this Windows API family is not registered yet; the plan records the requirement without pretending it is ready.",
            &format!("windows-api.{}", slug(&api.family)),
        );
    }

    for runtime in &profile.runtimes {
        let id = format!("runtime-{}", slug(&runtime.name));
        add_translation_step(
            &mut steps,
            &mut modules,
            &id,
            &format!("Prepare {}", runtime.name),
            "A runtime preparation executor is not registered yet; the requirement remains blocked until one exists.",
            &format!("runtime.{}", slug(&runtime.name)),
        );
    }

    if !profile.unresolved_imports.is_empty() {
        add_step(
            &mut steps,
            &mut modules,
            "resolve-dependencies",
            "Resolve bundled dependencies",
            "The analyzer found imports that are not present in the supplied game files. These must be resolved before a build can be considered complete.",
            "core.dependencies",
            StepStatus::Blocked,
        );
    }

    if !profile.protections.packers_or_protectors.is_empty() {
        add_step(
            &mut steps,
            &mut modules,
            "protection-review",
            "Review binary protection signals",
            "Record protection signals for planning and validation. No protection is modified or bypassed by this planning stage.",
            "analysis.protection",
            StepStatus::Ready,
        );
    }
    if !profile.protections.anti_cheats.is_empty() {
        add_step(
            &mut steps,
            &mut modules,
            "anti-cheat-review",
            "Review anti-cheat signals",
            "Record detected anti-cheat signals so the conversion plan can account for the title's requirements.",
            "analysis.anti_cheat",
            StepStatus::Ready,
        );
    }

    add_step(
        &mut steps,
        &mut modules,
        "build",
        "Build target application",
        "The target application builder is not executable yet, so Akron must not report a generated application.",
        "core.builder",
        StepStatus::Blocked,
    );
    add_step(
        &mut steps,
        &mut modules,
        "validate",
        "Validate generated application",
        "Launch/package validation cannot run until an executable build exists.",
        "core.validator",
        StepStatus::Blocked,
    );

    modules.sort();
    modules.dedup();
    AdaptationPlan {
        steps,
        required_modules: modules,
    }
}

fn add_translation_step(
    steps: &mut Vec<AdaptationStep>,
    modules: &mut Vec<String>,
    id: &str,
    title: &str,
    description: &str,
    module: &str,
) {
    add_step(
        steps,
        modules,
        id,
        title,
        description,
        module,
        StepStatus::Blocked,
    );
}

fn add_step(
    steps: &mut Vec<AdaptationStep>,
    modules: &mut Vec<String>,
    id: &str,
    title: &str,
    description: &str,
    module: &str,
    status: StepStatus,
) {
    modules.push(module.to_owned());
    steps.push(AdaptationStep {
        id: id.to_owned(),
        title: title.to_owned(),
        description: description.to_owned(),
        module: module.to_owned(),
        status,
    });
}

fn slug(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{StepStatus, build_plan};
    use akron_analyzer::profile::{
        ExecutableProfile, GameProfile, GraphicsRequirements, ProtectionSummary,
    };

    #[test]
    fn creates_game_specific_graphics_steps_without_faking_readiness() {
        let profile = GameProfile {
            executables: vec![ExecutableProfile {
                path: "game.exe".to_owned(),
                architecture: Some("x86_64".to_owned()),
                format: "PE".to_owned(),
            }],
            pe_binaries: Vec::new(),
            dependencies: Vec::new(),
            graphics: GraphicsRequirements {
                direct3d11: true,
                dxgi: true,
                ..GraphicsRequirements::default()
            },
            windows_apis: Vec::new(),
            runtimes: Vec::new(),
            unresolved_imports: Vec::new(),
            protections: ProtectionSummary::default(),
        };

        let plan = build_plan(&profile);
        assert!(
            plan.steps
                .iter()
                .any(|s| { s.id == "graphics-d3d11" && s.status == StepStatus::Blocked })
        );
        assert!(
            plan.steps
                .iter()
                .any(|s| { s.id == "graphics-dxgi" && s.status == StepStatus::Blocked })
        );
        assert!(
            plan.steps
                .iter()
                .any(|s| { s.id == "validate" && s.status == StepStatus::Blocked })
        );
    }

    #[test]
    fn unresolved_imports_block_dependency_resolution() {
        let profile = GameProfile {
            executables: Vec::new(),
            pe_binaries: Vec::new(),
            dependencies: Vec::new(),
            graphics: GraphicsRequirements::default(),
            windows_apis: Vec::new(),
            runtimes: Vec::new(),
            unresolved_imports: vec![akron_analyzer::profile::BinaryDependency {
                importer: "game.exe".to_owned(),
                library: "renderer.dll".to_owned(),
            }],
            protections: ProtectionSummary::default(),
        };

        let plan = build_plan(&profile);
        assert!(
            plan.steps
                .iter()
                .any(|s| { s.id == "resolve-dependencies" && s.status == StepStatus::Blocked })
        );
    }
}

