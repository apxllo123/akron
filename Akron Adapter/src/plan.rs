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
        "Review the detected executable, graphics, API, runtime, and protection requirements before changing files.",
        "core.analyzer",
        StepStatus::Ready,
    );

    if profile.graphics.direct3d9 {
        add_step(&mut steps, &mut modules, "graphics-d3d9", "Prepare Direct3D 9 → Metal", "Route Direct3D 9 rendering requirements through Akron's graphics translation pipeline.", "graphics.d3d9", StepStatus::Planned);
    }
    if profile.graphics.direct3d10 {
        add_step(&mut steps, &mut modules, "graphics-d3d10", "Prepare Direct3D 10 → Metal", "Route Direct3D 10 rendering requirements through Akron's graphics translation pipeline.", "graphics.d3d10", StepStatus::Planned);
    }
    if profile.graphics.direct3d11 {
        add_step(&mut steps, &mut modules, "graphics-d3d11", "Prepare Direct3D 11 → Metal", "Use the configured D3D11 translation backend and Metal-facing output path.", "graphics.d3d11", StepStatus::Planned);
    }
    if profile.graphics.direct3d12 {
        add_step(&mut steps, &mut modules, "graphics-d3d12", "Prepare Direct3D 12 → Metal", "Use the configured D3D12 translation backend and Metal-facing output path.", "graphics.d3d12", StepStatus::Planned);
    }
    if profile.graphics.dxgi {
        add_step(&mut steps, &mut modules, "graphics-dxgi", "Map DXGI requirements", "Prepare swap-chain, adapter, format, presentation, and related DXGI requirements for the target platform.", "graphics.dxgi", StepStatus::Planned);
    }
    if profile.graphics.vulkan {
        add_step(&mut steps, &mut modules, "graphics-vulkan", "Prepare Vulkan path", "Determine whether the game's Vulkan usage can be carried into the target graphics backend.", "graphics.vulkan", StepStatus::Planned);
    }
    if profile.graphics.opengl {
        add_step(&mut steps, &mut modules, "graphics-opengl", "Prepare OpenGL path", "Determine the required OpenGL-to-Metal transformation path for the detected workload.", "graphics.opengl", StepStatus::Planned);
    }

    for api in &profile.windows_apis {
        let id = format!("api-{}", slug(&api.family));
        add_step(&mut steps, &mut modules, &id, &format!("Map {} API family", api.family), "Generate a native implementation plan for the detected Windows API surface.", &format!("windows-api.{}", slug(&api.family)), StepStatus::Planned);
    }

    for runtime in &profile.runtimes {
        let id = format!("runtime-{}", slug(&runtime.name));
        add_step(&mut steps, &mut modules, &id, &format!("Prepare {}", runtime.name), "Resolve the detected runtime requirement into a native Akron-supported runtime strategy.", &format!("runtime.{}", slug(&runtime.name)), StepStatus::Planned);
    }

    if !profile.protections.packers_or_protectors.is_empty() {
        add_step(&mut steps, &mut modules, "protection-review", "Review binary protection signals", "Record protection signals for planning and validation. No protection is modified or bypassed by this planning stage.", "analysis.protection", StepStatus::Ready);
    }
    if !profile.protections.anti_cheats.is_empty() {
        add_step(&mut steps, &mut modules, "anti-cheat-review", "Review anti-cheat signals", "Record detected anti-cheat signals so the conversion plan can account for the title's requirements.", "analysis.anti_cheat", StepStatus::Ready);
    }

    add_step(&mut steps, &mut modules, "build", "Build target application", "Assemble the generated target-platform application from the selected modules.", "core.builder", StepStatus::Planned);
    add_step(&mut steps, &mut modules, "validate", "Validate generated application", "Check package structure, dependencies, architecture, permissions, and launch behavior before reporting success.", "core.validator", StepStatus::Planned);

    modules.sort();
    modules.dedup();
    AdaptationPlan {
        steps,
        required_modules: modules,
    }
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
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{StepStatus, build_plan};
    use akron_analyzer::profile::{ExecutableProfile, GameProfile, GraphicsRequirements, ProtectionSummary};

    #[test]
    fn creates_game_specific_graphics_steps() {
        let profile = GameProfile {
            executables: vec![ExecutableProfile {
                path: "game.exe".to_owned(),
                architecture: Some("x86_64".to_owned()),
                format: "PE".to_owned(),
            }],
            pe_binaries: Vec::new(),
            graphics: GraphicsRequirements {
                direct3d11: true,
                dxgi: true,
                ..GraphicsRequirements::default()
            },
            windows_apis: Vec::new(),
            runtimes: Vec::new(),
            protections: ProtectionSummary::default(),
        };

        let plan = build_plan(&profile);
        assert!(plan.steps.iter().any(|s| s.id == "graphics-d3d11"));
        assert!(plan.steps.iter().any(|s| s.id == "graphics-dxgi"));
        assert!(plan.steps.iter().any(|s| s.id == "validate" && s.status == StepStatus::Planned));
    }
}
