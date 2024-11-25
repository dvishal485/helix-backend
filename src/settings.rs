use serde::Serialize;
use serde_json::Value;

use crate::{cli_wrap::CliSetting, gio_wrap::GioSetting, modprobe_wrap::Modprobe,systemctl_wrap::Systemctl};

/* This trait is applicable on all
 * types of settings, it is used
 * to apply the settings
 */
pub trait ApplySettings {
    fn apply(self) -> Result<(), &'static str>;
    fn set_value(&mut self, value: Types);
}

#[derive(serde::Deserialize, Debug)]
/* POST request will have this structure
 * Frontend will only give us the id of
 * setting to be modified and the
 * modified value
 */
pub struct IncomingSettings {
    pub id: u32,
    pub value: Option<Types>,
}

/* Types of the value passed, this struct
* and its corresponding impl blocks are
* needed to parse the incoming value
* from the POST request
*/
#[derive(serde::Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum Types {
    Bool(bool),
    Int(i64),
    Double(f64),
    String(String),
}

impl From<Value> for Types {
    fn from(value: Value) -> Self {
        match value {
            Value::String(s) => Types::String(s),
            Value::Bool(b) => Types::Bool(b),
            Value::Number(n) if n.as_i64().is_none() => Types::Double(n.as_f64().unwrap()),
            Value::Number(n) => Types::Int(n.as_i64().unwrap()),
            _ => panic!("Expected string or boolean"),
        }
    }
}

impl Into<gio::glib::Variant> for Types {
    fn into(self) -> gio::glib::Variant {
        match self {
            Types::Bool(x) => x.into(),
            Types::Int(x) => x.into(),
            Types::Double(x) => x.into(),
            Types::String(x) => x.into(),
        }
    }
}

/* This struct is used to parse the
 * settings.json file
 */
#[derive(serde::Deserialize, Debug, Default, Clone)]
#[serde(untagged)]
pub enum SettingsType {
    GioSettings(GioSetting),
    ModProbe(Modprobe),
    CliSetting(CliSetting),
    Systemctl(Systemctl),
    #[default]
    Invalid,
}

impl ApplySettings for SettingsType {
    fn apply(self) -> Result<(), &'static str> {
        match self {
            SettingsType::GioSettings(x) => x.apply(),
            SettingsType::ModProbe(x) => x.apply(),
            Self::CliSetting(x) => x.apply(),
            SettingsType::Systemctl(x) => x.apply(),
            SettingsType::Invalid => Err("Invalid SettingsType"),
        }
    }

    fn set_value(&mut self, value: Types) {
        match self {
            SettingsType::GioSettings(x) => x.set_value(value),
            SettingsType::ModProbe(x) => x.set_value(value),
            Self::CliSetting(x) => x.set_value(value),
            SettingsType::Systemctl(x) => x.set_value(value),
            SettingsType::Invalid => (),
        }
    }
}
