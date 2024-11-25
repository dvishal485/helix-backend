use axum::{
    body::{boxed, Body, BoxBody},
    extract::State,
    http::{Request, Response, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::io::Write;
use std::{collections::HashMap, net::SocketAddr, process::Stdio};
use tower::ServiceExt;
use tower_http::services::ServeDir;
pub mod cli_wrap;
pub mod gio_wrap;
pub mod modprobe_wrap;
pub mod settings;
pub mod systemctl_wrap;
use settings::*;

fn construct_id_map() -> HashMap<u32, SettingsType> {
    let settings = serde_json::from_str::<Vec<Value>>(include_str!("../settings.json"));
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
                        let setting = schema_key_map
                            .get(schema)?
                            .iter()
                            .any(|x| key == *x)
                            .then(|| setting.into())?;
                        setting
                    }
                    SettingsType::ModProbe(setting) => {
                        setting.driver_exists().then(|| setting.into())?
                    }
                    SettingsType::Systemctl(setting) => {
                        setting.service_exists().then(|| setting.into())?
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

async fn handler(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    if uri.path() == "/" {
        return get_static_file("/index.html".parse::<Uri>().unwrap()).await;
    }

    let res = get_static_file(uri.clone()).await?;
    if res.status() == StatusCode::NOT_FOUND {
        match format!("{}.html", uri).parse() {
            Ok(uri_html) => get_static_file(uri_html).await,
            Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Invalid URI".to_string())),
        }
    } else {
        Ok(res)
    }
}

async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    match ServeDir::new("./build").oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}

fn prepare_terminal_with_sudo_access() {
    let mut proc = std::process::Command::new("sudo")
        .arg("-S")
        .arg("echo")
        .arg("'hello'")
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to run sudo command");

    let sudo_pass = std::env::var("SUDO_PASS").expect("SUDO_PASS not found in env");

    proc.stdin
        .take()
        .unwrap()
        .write_all(sudo_pass.as_bytes())
        .expect("Failed to write to stdin of sudo command");
}

#[tokio::main]
async fn main() {
    let settings_map: &HashMap<_, _> = Box::leak(Box::new(construct_id_map()));

    prepare_terminal_with_sudo_access();

    std::process::Command::new("xdg-open")
        .arg("http://localhost:3000")
        .spawn()
        .expect("Failed to open browser");

    let app = Router::new()
        .route("/set_config", post(set_config))
        .route("/get_all_configs", get(get_all_configs))
        .with_state(settings_map)
        .nest_service("/", get(handler));

    axum::Server::bind(&SocketAddr::from(([0, 0, 0, 0], 3000)))
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[test]
fn construct_id_map_test() {
    let settings_map = construct_id_map();
    eprintln!("{:?}", settings_map);
}

#[test]
fn static_variant_types() {
    use gio::prelude::*;
    eprintln!("{}", bool::static_variant_type().as_str());
    eprintln!("{}", i64::static_variant_type().as_str());
    eprintln!("{}", String::static_variant_type().as_str());
    eprintln!("{}", f64::static_variant_type().as_str());
}
