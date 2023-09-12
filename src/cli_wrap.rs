use crate::settings::ApplySettings;
use serde::Serialize;
use serde_json::Value;

#[derive(serde::Deserialize, Serialize, Debug)]
pub struct CliCommand {
    binary_name: String,
    args: String,
    superuser: bool,
}

// impl ApplySettings for CliCommand {
//     fn apply(&mut self) -> Result<(), &'static str>
//     {};
//     fn set_value(&mut self, value: Types){

//     };
// }
