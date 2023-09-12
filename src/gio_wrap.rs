use gio::{glib::FromVariant, prelude::*};
use std::collections::HashSet;

pub fn get_all_schema() -> HashSet<String> {
    gio::SettingsSchemaSource::default()
        .iter()
        .map(|x| x.list_schemas(true))
        .flat_map(|(settings1, settings2)| settings1.into_iter().chain(settings2.into_iter()))
        .map(|x| x.to_string())
        .collect()
}

pub fn get_all_keys_from_schema(
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

pub fn get_key_from_schema<T: FromVariant>(
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

pub fn set_key_from_schema<T: Into<gio::glib::Variant>>(
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
