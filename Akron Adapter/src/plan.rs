use akron_analyzer::{BinaryDependency, GameProfile};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Ready,
    Blocked,
    Planned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanStep {
    pub id: &'static str,
    pub status: StepStatus,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyResolutionKind {
    BundledGame,
    WindowsPlatform,
    KnownRuntime,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyResolution {
    pub dependency: BinaryDependency,
    pub kind: DependencyResolutionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdaptationPlan {
    pub steps: Vec<PlanStep>,
    pub dependency_resolutions: Vec<DependencyResolution>,
}

pub fn build_plan(profile: &GameProfile) -> AdaptationPlan {
    let dependency_resolutions = resolve_dependencies(profile);
    let dependency_blocked = dependency_resolutions
        .iter()
        .any(|resolution| resolution.kind == DependencyResolutionKind::Unresolved);

    let mut steps = Vec::new();
    steps.push(PlanStep {
        id: "resolve-dependencies",
        status: if dependency_blocked {
            StepStatus::Blocked
        } else {
            StepStatus::Ready
        },
        reason: if dependency_blocked {
            "One or more imported libraries could not be resolved.".to_owned()
        } else {
            "All imported libraries are accounted for.".to_owned()
        },
    });

    steps.push(PlanStep {
        id: "convert-graphics",
        status: StepStatus::Blocked,
        reason: "Graphics conversion backends are not implemented yet.".to_owned(),
    });

    steps.push(PlanStep {
        id: "build-adapted-app",
        status: StepStatus::Blocked,
        reason: "Native application build pipeline is not implemented yet.".to_owned(),
    });

    steps.push(PlanStep {
        id: "validate",
        status: StepStatus::Blocked,
        reason: "Validation pipeline is not implemented yet.".to_owned(),
    });

    AdaptationPlan {
        steps,
        dependency_resolutions,
    }
}

fn resolve_dependencies(profile: &GameProfile) -> Vec<DependencyResolution> {
    profile
        .dependencies
        .iter()
        .cloned()
        .map(|dependency| {
            let library = dependency.library.to_ascii_lowercase();
            let kind = if profile
                .pe_binaries
                .iter()
                .any(|binary| binary.path.to_ascii_lowercase().ends_with(&library))
            {
                DependencyResolutionKind::BundledGame
            } else if is_windows_platform_library(&library) {
                DependencyResolutionKind::WindowsPlatform
            } else if is_known_runtime(&library) {
                DependencyResolutionKind::KnownRuntime
            } else {
                DependencyResolutionKind::Unresolved
            };

            DependencyResolution { dependency, kind }
        })
        .collect()
}

fn is_windows_platform_library(library: &str) -> bool {
    matches!(
        library,
        "kernel32.dll"
            | "kernelbase.dll"
            | "ntdll.dll"
            | "user32.dll"
            | "advapi32.dll"
            | "gdi32.dll"
            | "gdi32full.dll"
            | "ole32.dll"
            | "oleaut32.dll"
            | "shell32.dll"
            | "comdlg32.dll"
            | "ws2_32.dll"
            | "winhttp.dll"
            | "winmm.dll"
            | "version.dll"
            | "shlwapi.dll"
            | "secur32.dll"
            | "bcrypt.dll"
            | "crypt32.dll"
    )
}

fn is_known_runtime(library: &str) -> bool {
    matches!(
        library,
        "msvcp140.dll"
            | "vcruntime140.dll"
            | "vcruntime140_1.dll"
            | "ucrtbase.dll"
            | "concrt140.dll"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use akron_analyzer::{
        ExecutableProfile, GraphicsRequirements, PeBinaryProfile, ProtectionSummary,
        RuntimeRequirement, WindowsApiRequirement,
    };

    fn base_profile() -> GameProfile {
        GameProfile {
            executables: vec![ExecutableProfile {
                path: "game.exe".to_owned(),
                architecture: "x86_64".to_owned(),
            }],
            pe_binaries: vec![PeBinaryProfile {
                path: "renderer.dll".to_owned(),
                architecture: "x86_64".to_owned(),
                kind: "dll".to_owned(),
                import_count: 0,
                libraries: Vec::new(),
            }],
            dependencies: Vec::new(),
            graphics: GraphicsRequirements::default(),
            windows_apis: Vec::<WindowsApiRequirement>::new(),
            runtimes: Vec::<RuntimeRequirement>::new(),
            unresolved_imports: Vec::new(),
            protections: ProtectionSummary::default(),
        }
    }

    #[test]
    fn bundled_dependency_is_resolved() {
        let mut profile = base_profile();
        profile.dependencies = vec![BinaryDependency {
            importer: "game.exe".to_owned(),
            library: "renderer.dll".to_owned(),
        }];

        let plan = build_plan(&profile);
        assert_eq!(
            plan.dependency_resolutions[0].kind,
            DependencyResolutionKind::BundledGame
        );
    }

    #[test]
    fn platform_dependency_is_resolved() {
        let mut profile = base_profile();
        profile.dependencies = vec![BinaryDependency {
            importer: "game.exe".to_owned(),
            library: "kernel32.dll".to_owned(),
        }];

        let plan = build_plan(&profile);
        assert_eq!(
            plan.dependency_resolutions[0].kind,
            DependencyResolutionKind::WindowsPlatform
        );
    }

    #[test]
    fn known_runtime_dependency_is_resolved() {
        let mut profile = base_profile();
        profile.dependencies = vec![BinaryDependency {
            importer: "game.exe".to_owned(),
            library: "vcruntime140.dll".to_owned(),
        }];

        let plan = build_plan(&profile);
        assert_eq!(
            plan.dependency_resolutions[0].kind,
            DependencyResolutionKind::KnownRuntime
        );
    }

    #[test]
    fn unresolved_dependency_blocks_dependency_step() {
        let mut profile = base_profile();
        profile.dependencies = vec![BinaryDependency {
            importer: "game.exe".to_owned(),
            library: "missing.dll".to_owned(),
        }];

        let plan = build_plan(&profile);
        assert_eq!(
            plan.dependency_resolutions[0].kind,
            DependencyResolutionKind::Unresolved
        );
        assert!(plan.steps.iter().any(|step| {
            step.id == "resolve-dependencies" && step.status == StepStatus::Blocked
        }));
    }

    #[test]
    fn resolved_dependencies_make_dependency_step_ready() {
        let mut profile = base_profile();
        profile.dependencies = vec![
            BinaryDependency {
                importer: "game.exe".to_owned(),
                library: "renderer.dll".to_owned(),
            },
            BinaryDependency {
                importer: "game.exe".to_owned(),
                library: "kernel32.dll".to_owned(),
            },
        ];

        let plan = build_plan(&profile);
        assert!(
            plan.steps.iter().any(|step| {
                step.id == "resolve-dependencies" && step.status == StepStatus::Ready
            })
        );
    }

    #[test]
    fn unresolved_imports_block_dependency_resolution() {
        let mut profile = base_profile();
        profile.unresolved_imports = vec![BinaryDependency {
            importer: "game.exe".to_owned(),
            library: "renderer.dll".to_owned(),
        }];
        profile.dependencies = profile.unresolved_imports.clone();

        let plan = build_plan(&profile);
        assert!(plan.steps.iter().any(|step| {
            step.id == "resolve-dependencies" && step.status == StepStatus::Blocked
        }));
    }
}
