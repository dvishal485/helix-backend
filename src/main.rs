/*
 * I know the code is dirty, and I'm sorry about it.
 * I am an average developer, and I'm still learning.
 * Also I'm sleepy, so I'm not gonna clean it up.
 * Thanks for understanding.
 */
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::{collections::HashMap, net::SocketAddr};

pub mod cli_wrap;
pub mod gio_wrap;
pub mod modprobe_wrap;
pub mod settings;
use settings::*;

fn construct_id_map() -> HashMap<u32, SettingsType> {
    let settings = serde_json::from_str::<Vec<Value>>(
        // &std::fs::read_to_string("settings.json").expect("settings.json not found"),
        include_str!("../settings.json"),
    );
    settings
        .expect("Couldn't construct settings map from settings.json")
        .into_iter()
        .map(|setting| {
            (
                setting["id"]
                    .as_u64()
                    .expect("One of setting['id'] not a valid u64.") as u32,
                serde_json::from_value::<SettingsType>(setting).unwrap(),
            )
        })
        .collect::<HashMap<u32, SettingsType>>()
}

async fn set_config(
    State(all_settings): State<&HashMap<u32, SettingsType>>,
    Json(body): Json<IncomingSettings>,
) -> impl IntoResponse {
    let Some(value) = body.value else {
        return (StatusCode::BAD_REQUEST, "value not found");
    };
    let Some(mut setting) = all_settings.get(&body.id).cloned() else {
        return (StatusCode::BAD_REQUEST, "setting not found");
    };

    setting.set_value(value);

    if let Err(e) = setting.apply() {
        return (StatusCode::BAD_REQUEST, e);
    }

    (StatusCode::OK, "ok")
}

async fn get_all_configs(
    State(all_settings): State<&HashMap<u32, SettingsType>>,
) -> impl IntoResponse {
    let system_gio_schemas = gio_wrap::get_all_schema();

    let schema_key_map = gio_wrap::get_schema_key_map(system_gio_schemas);

    let matched_gio_settings: Vec<(u32, Types)> = all_settings
        .into_iter()
        .filter_map(|(&id, setting)| {
            Some((
                id,
                match setting {
                    SettingsType::GioSettings(setting) => {
                        let schema = setting.schema.as_str();
                        let key = setting.key.as_str();
                        let setting = schema_key_map.get(schema).and_then(|keys| {
                            keys.iter().find(|x| key == *x).map(|_| setting.into())
                        })?;
                        setting
                    }
                    SettingsType::ModProbe(setting) => {
                        if setting.driver_exists() {
                            setting.into()
                        } else {
                            return None;
                        }
                    }
                    SettingsType::CliSetting(_) => {
                        // @TODO: implement a checker fn to see curr state of the cli setting
                        // probably by make new fields with commands to check state
                        return None;
                    }
                    SettingsType::Invalid => return None,
                },
            ))
        })
        .collect();

    (StatusCode::OK, json!(matched_gio_settings).to_string())
}

#[tokio::main]
async fn main() {
    let settings_map: &HashMap<_, _> = Box::leak(Box::new(construct_id_map()));

    let app = Router::new()
        .route("/set_config", post(set_config))
        .route("/get_all_config", get(get_all_configs))
        .with_state(settings_map);

    _ = axum::Server::bind(&SocketAddr::from(([0, 0, 0, 0], 3000)))
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[test]
fn construct_id_map_test() {
    let settings_map = construct_id_map();
    eprintln!("{:?}", settings_map);
}
