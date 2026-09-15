//! Sub-plugin modules. Add or remove entries in `build_registry` only.

pub mod board;
pub mod config;
pub mod docs;
pub mod doctor;
pub mod git;
pub mod launch;
pub mod loop_ctl;
pub mod master;
pub mod roles;
pub mod secrets;

use crate::registry::PluginRegistry;

/// Single place to enable/disable sub-plugins.
pub fn build_registry() -> PluginRegistry {
    let mut reg = PluginRegistry::new();
    reg.register(Box::new(docs::DocsPlugin::new()));
    reg.register(Box::new(master::MasterPlugin::new()));
    reg.register(Box::new(launch::LaunchPlugin::new()));
    reg.register(Box::new(loop_ctl::LoopPlugin::new()));
    reg.register(Box::new(roles::RolesPlugin::new()));
    reg.register(Box::new(git::GitPlugin::new()));
    reg.register(Box::new(board::BoardPlugin::new()));
    reg.register(Box::new(secrets::SecretsPlugin::new()));
    reg.register(Box::new(config::ConfigPlugin::new()));
    reg.register(Box::new(doctor::DoctorPlugin::new()));
    reg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_expected_plugins_registered() {
        let reg = build_registry();
        let ids = reg.ids();
        for expected in [
            "docs",
            "master",
            "launch",
            "loop",
            "roles",
            "git",
            "board",
            "secrets",
            "config",
            "doctor",
        ] {
            assert!(ids.contains(&expected), "missing {expected}");
        }
        assert_eq!(ids.len(), 10);
    }
}
