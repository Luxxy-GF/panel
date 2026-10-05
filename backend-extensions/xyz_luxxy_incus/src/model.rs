use anyhow::{Context, ensure};
use serde::{Deserialize, Serialize};
use shared::models::{ModelExtension, SafeModelExtension};
use sqlx::{Row, postgres::PgRow};
use std::collections::BTreeMap;
use utoipa::ToSchema;

pub const OS_EGG: uuid::Uuid = uuid::uuid!("7f9047ea-14c8-4f8f-a1de-f267ea740111");

pub fn require(
    condition: bool,
    message: &'static str,
) -> Result<(), shared::database::DatabaseError> {
    if condition {
        Ok(())
    } else {
        Err(anyhow::anyhow!(message).into())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstanceKind {
    Container,
    VirtualMachine,
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeInstance {
    pub kind: InstanceKind,
    pub image: String,
    #[serde(default)]
    pub config: BTreeMap<String, String>,
}

impl NativeInstance {
    pub fn without_config(mut self) -> Self {
        self.config.clear();
        self
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            !self.image.is_empty()
                && self.image.len() <= 255
                && !self.image.bytes().any(|byte| byte.is_ascii_control()),
            "invalid OS image alias"
        );
        validate_config(self)?;
        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ServerData {
    pub instance: Option<NativeInstance>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ApiMetadata {
    pub incus_instance: Option<NativeInstance>,
}

pub struct ServerExtension;

impl SafeModelExtension for ServerExtension {
    type Value = ServerData;
    fn name() -> &'static str {
        "xyz.luxxy.incus"
    }
}

impl ModelExtension for ServerExtension {
    fn extension_name(&self) -> &'static str {
        "xyz.luxxy.incus"
    }

    fn extended_columns(&self, prefix: &str) -> BTreeMap<&'static str, compact_str::CompactString> {
        BTreeMap::from([(
            "servers.xyz_luxxy_incus_instance",
            compact_str::format_compact!("{prefix}xyz_luxxy_incus_instance"),
        )])
    }

    fn map_extended(
        &self,
        prefix: &str,
        row: &PgRow,
    ) -> Result<shared::models::ModelExtensionMapType, shared::database::DatabaseError> {
        let value: Option<serde_json::Value> =
            row.try_get(compact_str::format_compact!("{prefix}xyz_luxxy_incus_instance").as_str())?;
        Ok(Box::new(ServerData {
            instance: value
                .map(serde_json::from_value)
                .transpose()
                .context("invalid stored Incus instance metadata")?,
        }))
    }
}

tokio::task_local! {
    pub static CREATING_INSTANCE: NativeInstance;
    pub static UPDATING_INSTANCE: NativeInstance;
}

#[derive(Deserialize)]
struct ConfigOption {
    key: String,
    #[serde(rename = "type")]
    value_type: String,
    kinds: Vec<String>,
    managed: bool,
    choices: Vec<String>,
}

fn validate_config(instance: &NativeInstance) -> anyhow::Result<()> {
    static OPTIONS: std::sync::LazyLock<Vec<ConfigOption>> = std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("../frontend/src/config-options.json"))
            .expect("invalid Incus option catalog")
    });
    ensure!(
        instance.config.len() <= 128,
        "too many Incus instance options"
    );
    ensure!(
        instance
            .config
            .iter()
            .map(|(key, value)| key.len() + value.len())
            .sum::<usize>()
            <= 524_288,
        "Incus instance configuration exceeds 512 KiB"
    );
    let kind = match instance.kind {
        InstanceKind::Container => "container",
        InstanceKind::VirtualMachine => "virtual_machine",
    };
    for (key, value) in &instance.config {
        ensure!(
            key.len() <= 255
                && !key.starts_with("user.wings.")
                && key
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')),
            "invalid or reserved Incus option: {key}"
        );
        let option = OPTIONS
            .iter()
            .find(|option| {
                option.key == *key
                    || option
                        .key
                        .strip_suffix('*')
                        .is_some_and(|prefix| key.starts_with(prefix) && key.len() > prefix.len())
            })
            .with_context(|| format!("unsupported Incus 7.0 instance option: {key}"))?;
        ensure!(
            !option.managed,
            "{key} is managed by Wings resource or lifecycle settings"
        );
        ensure!(
            option.kinds.iter().any(|value| value == kind),
            "{key} is not supported for {kind}"
        );
        ensure!(
            !value.is_empty() && value.len() <= 65_536 && !value.contains('\0'),
            "invalid value for {key}"
        );
        match option.value_type.as_str() {
            "bool" => ensure!(
                matches!(value.as_str(), "true" | "false"),
                "{key} requires true or false"
            ),
            "integer" | "int64" => {
                value
                    .parse::<i64>()
                    .with_context(|| format!("{key} requires an integer"))?;
            }
            _ => (),
        }
        ensure!(
            option.choices.is_empty() || option.choices.contains(value),
            "invalid choice for {key}"
        );
    }
    Ok(())
}
