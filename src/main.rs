use std::{
    collections::{HashMap, HashSet},
    io::Read,
};

use axum::{routing::get, Router};
use gio::{
    glib::{FromVariant, GString, Type},
    prelude::*,
};

fn main() {
    test();
}

/* #[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
} */

fn test() {
    println!("{:?}", get_all_schema());
    println!(
        "{:?}",
        get_all_keys_from_schema(get_all_schema(), "com.ubuntu.login-screen")
    );
    println!(
        "{:?}",
        get_key_from_schema::<String>(
            get_all_keys_from_schema(get_all_schema(), "com.ubuntu.login-screen").unwrap(),
            "com.ubuntu.login-screen",
            "background-size"
        )
    );
}

fn get_all_schema() -> HashSet<String> {
    let mut schemas = HashSet::new();
    gio::SettingsSchemaSource::default()
        .iter()
        .map(|x| x.list_schemas(true))
        .for_each(|(settings1, settings2)| {
            schemas.extend(
                settings1
                    .into_iter()
                    .chain(settings2.into_iter())
                    .map(|x| x.to_string()),
            )
        });
    schemas
}

fn get_all_keys_from_schema(
    available_schemas: HashSet<String>,
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
    available_keys: HashSet<String>,
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
    available_keys: HashSet<String>,
    schema: &str,
    key: &str,
    value: T,
) -> Result<(), ()> {
    if available_keys.contains(key) {
        let setting = gio::Settings::new(schema);
        setting.set(key, value).map_err(|_| ())
    } else {
        Err(())
    }
}
