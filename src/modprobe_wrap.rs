use crate::settings::ApplySettings;
use crate::settings::Types;
use serde::Serialize;
use serde_json::Value;
use std::process::Command;

#[derive(serde::Deserialize, Serialize, Debug, Clone)]
pub struct Modprobe {
    pub driver: String,
    pub enable: Option<bool>,
}

impl ApplySettings for Modprobe {
    fn apply(&mut self) -> Result<(), &'static str> {
        let Some(enabled) = self.enable else {
            return Err("enabled is not set");
        };
        let mut cmd = Command::new("sudo");
        cmd.arg("modprobe");
        if !enabled {
            cmd.arg("-r");
        }
        cmd.arg(self.driver.clone());
        match cmd.output() {
            Ok(output) => Ok(()),
            Err(e) => {
                eprintln!("{e}");
                Err("Failed to run modprobe")
            }
        }
    }
    fn set_value(&mut self, value: crate::settings::Types) {
        match value {
            Types::Bool(b) => self.enable = Some(b),
            _ => panic!("Invalid type for Modprobe"),
        }
    }
}
