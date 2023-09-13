use crate::settings::ApplySettings;
use crate::settings::Types;
use serde::Serialize;
use std::process::Command;

#[derive(serde::Deserialize, Serialize, Debug, Clone)]
pub struct Modprobe {
    pub driver: String,
    pub enable: Option<bool>,
}

impl ApplySettings for Modprobe {
    fn apply(self) -> Result<(), &'static str> {
        let Some(enable) = self.enable else {
            return Err("enabled is not set");
        };
        if enable {
            self.enable_driver()
        } else {
            self.disable_driver()
        }
    }
    fn set_value(&mut self, value: crate::settings::Types) {
        match value {
            Types::Bool(b) => self.enable = Some(b),
            _ => panic!("Invalid type for Modprobe"),
        }
    }
}

impl Into<Types> for &Modprobe {
    fn into(self) -> Types {
        Types::Bool(self.driver_state())
    }
}

impl Modprobe {
    pub fn driver_exists(&self) -> bool {
        let mut cmd = Command::new("modprobe");
        cmd.arg("-n");
        cmd.arg(self.driver.clone());
        match cmd.output() {
            Ok(output) => output.status.success(),
            Err(e) => {
                eprintln!("{e}");
                false
            }
        }
    }
    pub fn driver_state(&self) -> bool {
        let mut cmd = Command::new("lsmod");
        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    let output = String::from_utf8(output.stdout).unwrap();
                    output.contains(&self.driver)
                } else {
                    eprintln!("lsmod failed");
                    false
                }
            }
            Err(e) => {
                eprintln!("{e}");
                false
            }
        }
    }
    pub fn disable_driver(self) -> Result<(), &'static str> {
        let mut cmd = Command::new("sudo");
        cmd.arg("modprobe");
        cmd.arg("-r");
        cmd.arg(self.driver);
        match cmd.output() {
            Ok(output) => output
                .status
                .success()
                .then(|| ())
                .ok_or("Failed to remove driver"),
            Err(e) => {
                eprintln!("{e}");
                Err("Failed to run modprobe")
            }
        }
    }
    pub fn enable_driver(self) -> Result<(), &'static str> {
        let mut cmd = Command::new("sudo");
        cmd.arg("modprobe");
        cmd.arg(self.driver);
        match cmd.output() {
            Ok(output) => output
                .status
                .success()
                .then(|| ())
                .ok_or("Failed to enable driver"),
            Err(e) => {
                eprintln!("{e}");
                Err("Failed to run modprobe")
            }
        }
    }
}

#[test]
fn modprobe_driver_exists() {
    let modprobe = Modprobe {
        driver: "idonotexists".to_string(),
        enable: None,
    };
    assert!(!modprobe.driver_exists());

    let modprobe = Modprobe {
        driver: "uvcvideo".to_string(),
        enable: None,
    };
    assert!(modprobe.driver_exists());
}
