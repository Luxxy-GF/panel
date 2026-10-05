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
}

impl NativeInstance {
    pub fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            !self.image.is_empty()
                && self.image.len() <= 255
                && !self.image.bytes().any(|byte| byte.is_ascii_control()),
            "invalid OS image alias"
        );
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
}
