use crate::settings::ApplySettings;
use crate::settings::Types;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Systemctl {
    service_name: String,
    enable: Option<bool>,
}

impl ApplySettings for Systemctl {
    fn apply(self) -> Result<(), &'static str> {
        let Some(enable) = self.enable else {
            return Err("Enabled is not set");
        };
        if enable {
            self.enable_service()
        } else {
            self.disable_service()
        }
    }

    fn set_value(&mut self, value: crate::settings::Types) {
        match value {
            Types::Bool(b) => self.enable = Some(b),
            _ => panic!("Invalid type for Systemctl"),
        }
    }
}

impl Into<Types> for &Systemctl {
    fn into(self) -> Types {
        Types::Bool(self.service_state())
    }
}

impl Systemctl {
    pub fn service_exists(&self) -> bool {
        let mut cmd = Command::new("systemctl");
        match cmd.output() {
            Ok(output) => {
                //  println!("{:?}", String::from_utf8(output.stdout).unwrap());
                std::str::from_utf8(&output.stdout)
                    .expect("Couldn't convert output of cmd to UTF-8 string")
                    .contains(self.service_name.as_str())
            }
            Err(e) => {
                eprintln!("{e}");
                false
            }
        }
    }

    pub fn service_state(&self) -> bool {
        let mut cmd: Command = Command::new("systemctl");
        cmd.arg("is-active");
        match cmd.output() {
            Ok(output) => {
                std::str::from_utf8(&output.stdout)
                    .expect("Couldn't convert output of cmd to UTF-8 string")
                    == "0"
            }
            Err(e) => {
                eprintln!("{e}");
                false
            }
        }
    }

    pub fn enable_service(self) -> Result<(), &'static str> {
        let mut cmd = Command::new("sudo");
        cmd.arg("systemctl");
        cmd.arg("start");
        cmd.arg(self.service_name.as_str());
        match cmd.output() {
            Ok(output) => output
                .status
                .success()
                .then(|| ())
                .ok_or("Failed to enable service"),
            Err(e) => {
                eprintln!("{e}");
                Err("Failed to run systemctl")
            }
        }
    }

    pub fn disable_service(self) -> Result<(), &'static str> {
        let mut cmd = Command::new("sudo");
        cmd.arg("systemctl");
        cmd.arg("stop");
        cmd.arg(self.service_name.as_str());
        match cmd.output() {
            Ok(output) => output
                .status
                .success()
                .then(|| ())
                .ok_or("Failed to disable service"),
            Err(e) => {
                eprintln!("{e}");
                Err("Failed to run systemctl")
            }
        }
    }
}

#[test]
fn systemctl_service_exists() {
    let systemctl = Systemctl {
        service_name: "idontexist".to_string(),
        enable: None,
    };
    println!("{}", systemctl.service_exists());
    assert!(!systemctl.service_exists());
}
