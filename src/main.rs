use std::{
    collections::{HashMap, HashSet},
    io::Read,
    net::SocketAddr,
};

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use gio::{
    glib::{FromVariant, GString, Type},
    prelude::*,
};

async fn set_config(Json(body): Json<HashMap<String, String>>) -> impl IntoResponse {
    let schema = body.get("schema").unwrap();
    let key = body.get("key").unwrap();
    let value = format!("{schema} {key}");
    (StatusCode::OK, value)
    // let schemas = get_all_schema();
    // let settings = get_all_keys_from_schema(&schemas, schema).expect("schema not found");
    // let apply_settings = set_key_from_schema(&settings, schema, key, value);
    // match apply_settings {
    //     Ok(_) => format!("{}: {} set to {}", schema, key, value),
    //     Err(_) => format!("{}: {} not found", schema, key),
    // }
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    // with a POST route which receives a JSON body
    let app = Router::new().route("/set_config", post(set_config));

    // run it with hyper on localhost:3000
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