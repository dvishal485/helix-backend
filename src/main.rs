/*
 * I know the code is dirty, and I'm sorry about it.
 * I am an average developer, and I'm still learning.
 * Also I'm sleepy, so I'm not gonna clean it up.
 * Thanks for understanding.
 */
use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use gio::{glib::FromVariant, prelude::*};
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
};

#[derive(serde::Deserialize, Debug)]
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

#[derive(serde::Deserialize, Debug)]
pub struct IncomingSettings {
    id: u32,
    value: Option<Types>,
}

pub trait ApplySettings {
    fn apply(&mut self) -> Result<(), &'static str>;
    fn set_value(&mut self, value: Types);
}

#[derive(serde::Deserialize, Debug, Default)]
#[serde(untagged)]
pub enum SettingsType {
    GioSettings(GioSetting),
    #[default]
    Invalid,
}

#[derive(serde::Deserialize, Debug)]
pub struct GioSetting {
    schema: String,
    key: String,
    value: Option<Types>,
}

impl ApplySettings for SettingsType {
    fn apply(&mut self) -> Result<(), &'static str> {
        match self {
            SettingsType::GioSettings(x) => x.apply(),
            SettingsType::Invalid => Err("Invalid SettingsType"),
        }
    }

    fn set_value(&mut self, value: Types) {
        match self {
            SettingsType::GioSettings(x) => x.set_value(value),
            SettingsType::Invalid => (),
        }
    }
}

impl ApplySettings for GioSetting {
    fn apply(&mut self) -> Result<(), &'static str> {
        let setting = gio::Settings::new(&self.schema);
        let value = self.value.take().unwrap();
        if let Ok(_) = setting.set(self.key.as_str(), value) {
            setting.apply();
            gio::Settings::sync();
            Ok(())
        } else {
            Err("Settings couldn't be applied.")
        }
    }

    fn set_value(&mut self, value: Types) {
        let value = value.into();
        self.value = Some(value);
    }
}

fn construct_id_map() -> HashMap<u32, SettingsType> {
    let settings = serde_json::from_str::<Vec<Value>>(
        // &std::fs::read_to_string("settings.json").expect("settings.json not found"),
        include_str!("../settings.json"),
    );
    settings
        .unwrap()
        .into_iter()
        .map(|setting| {
            (
                setting["id"].as_u64().unwrap() as u32,
                serde_json::from_value::<SettingsType>(setting.clone()).unwrap(),
            )
        })
        .collect::<HashMap<u32, SettingsType>>()
}

async fn set_config(Json(mut body): Json<IncomingSettings>) -> impl IntoResponse {
    let mut settings_map = construct_id_map();
    let Some(value) = body.value.take() else {
        return (StatusCode::BAD_REQUEST, "value not found");
    };
    let Some(setting) = settings_map.get_mut(&body.id) else {
        return (StatusCode::BAD_REQUEST, "setting not found");
    };

    let mut setting = std::mem::take(setting);
    setting.set_value(value);

    if let Err(e) = setting.apply() {
        return (StatusCode::BAD_REQUEST, e);
    }

    (StatusCode::OK, "ok")
}

#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
pub enum SettingsAvailable {
    GioSettings(GioSetting),
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/set_config", post(set_config));
    // .route("/set_config", get(get_configs));

    axum::Server::bind(&SocketAddr::from(([0, 0, 0, 0], 3000)))
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn test() {
    let schema = "org.gnome.desktop.sound";
    let key = "allow-volume-above-100-percent";
    let schemas = get_all_schema();
    let settings = get_all_keys_from_schema(&schemas, schema).expect("schema not found");
    println!("{:?}", get_key_from_schema::<bool>(&settings, schema, key));
    let apply_settings = set_key_from_schema(&settings, schema, key, false);
    println!("{:?}", apply_settings);
    println!("{:?}", get_key_from_schema::<bool>(&settings, schema, key));
}

fn get_all_schema() -> HashSet<String> {
    gio::SettingsSchemaSource::default()
        .iter()
        .map(|x| x.list_schemas(true))
        .flat_map(|(settings1, settings2)| settings1.into_iter().chain(settings2.into_iter()))
        .map(|x| x.to_string())
        .collect()
}

fn get_all_keys_from_schema(
    available_schemas: &HashSet<String>,
    schema: &str,
) -> Option<HashSet<String>> {
    if available_schemas.contains(schema) {
        let setting = gio::Settings::new(schema);
        Some(HashSet::from_iter(
            setting
                .settings_schema()
                .unwrap()
                .list_keys()
                .into_iter()
                .map(|x| x.to_string()),
        ))
    } else {
        None
    }
}

fn get_key_from_schema<T: FromVariant>(
    available_keys: &HashSet<String>,
    schema: &str,
    key: &str,
) -> Result<T, ()> {
    if available_keys.contains(key) {
        let setting = gio::Settings::new(schema);
        let value = setting.get::<T>(key);
        Ok(value)
    } else {
        Err(())
    }
}

fn set_key_from_schema<T: Into<gio::glib::Variant>>(
    available_keys: &HashSet<String>,
    schema: &str,
    key: &str,
    value: T,
) -> Result<(), &'static str> {
    if available_keys.contains(key) {
        let setting = gio::Settings::new(schema);
        if let Ok(_) = setting.set(key, value) {
            setting.apply();
            gio::Settings::sync();
            Ok(())
        } else {
            Err("Settings couldn't be applied.")
        }
    } else {
        Err("Key not found. Ensure you pass HashSet of Keys.")
    }
}
