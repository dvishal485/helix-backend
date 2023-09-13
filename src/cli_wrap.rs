use crate::settings::ApplySettings;
use crate::settings::Types;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RunnerProgram {
    #[serde(rename = "bash")]
    Bash,
    #[serde(rename = "shell")]
    ShellScript,
}

impl Into<&'static str> for RunnerProgram {
    fn into(self) -> &'static str {
        match self {
            RunnerProgram::Bash => "/usr/bin/bash",
            RunnerProgram::ShellScript => "/usr/bin/sh",
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CliSetting {
    pub terminal: RunnerProgram,
    pub command: String,
}

impl ApplySettings for CliSetting {
    fn apply(self) -> Result<(), &'static str> {
        let mut cmd = Command::new::<&str>(self.terminal.into());
        cmd.arg("-c");
        cmd.arg(self.command);
        match cmd.output() {
            Ok(output) => {
                if output.status.success() {
                    println!("Command succeeded: {:?}", output);
                    Ok(())
                } else {
                    eprintln!("Command failed: {:?}", output);
                    Err("Command failed")
                }
            }
            Err(e) => {
                eprintln!("{e}");
                Err("Command failed")
            }
        }
    }

    fn set_value(&mut self, value: Types) {
        match value {
            Types::String(s) => self.command = s,
            _ => panic!("Invalid type for CliSetting"),
        };
    }
}

#[test]
fn test_cli_setting() {
    let mut setting = CliSetting {
        command: "echo 1".to_string(),
        terminal: RunnerProgram::Bash,
    };
    let command = "echo 2 && (echo 3 | grep 3)";
    setting.set_value(Types::String(command.to_string()));
    assert_eq!(setting.command, command);
    assert!(setting.clone().apply().is_ok());

    setting.terminal = RunnerProgram::ShellScript;
    assert_eq!(setting.command, command);
    assert!(setting.apply().is_ok());
}

#[test]
fn serialize_cli_settings() {
    let setting = CliSetting {
        command: "echo 1".to_string(),
        terminal: RunnerProgram::ShellScript,
    };
    let json = serde_json::to_string(&setting).expect("Couldn't serialize CliSetting");
    eprintln!("{:?}", json);
    assert!(json.contains("shell"));
}
