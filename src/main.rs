/*
 * I know the code is dirty, and I'm sorry about it.
 * I am an average developer, and I'm still learning.
 * Also I'm sleepy, so I'm not gonna clean it up.
 * Thanks for understanding.
 */
use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde_json::Value;
use std::{collections::HashMap, net::SocketAddr};
pub mod gio_wrap;
pub mod settings;
use settings::*;

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

#[tokio::main]
async fn main() {
    let app = Router::new().route("/set_config", post(set_config));
    // .route("/set_config", get(get_configs));

    axum::Server::bind(&SocketAddr::from(([0, 0, 0, 0], 3000)))
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[test]
fn test() {
    use gio_wrap::*;
    let schema = "org.gnome.desktop.sound";
    let key = "allow-volume-above-100-percent";
    let schemas = get_all_schema();
    let settings = get_all_keys_from_schema(&schemas, schema).expect("schema not found");
    set_key_from_schema(&settings, schema, key, false).expect("key not found");
}
