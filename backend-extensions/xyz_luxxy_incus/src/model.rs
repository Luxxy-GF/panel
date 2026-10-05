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
    #[serde(default)]
    pub network: Option<ExternalNetwork>,
}

impl NativeInstance {
    pub fn without_config(mut self) -> Self {
        self.config.clear();
        self.network = None;
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
        if let Some(network) = &self.network {
            network.validate()?;
            ensure!(
                !self.config.contains_key("cloud-init.network-config")
                    && !self.config.contains_key("user.network-config"),
                "IP pools manage guest network configuration"
            );
        }
        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ServerData {
    pub instance: Option<NativeInstance>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ApiMetadata {
    #[schema(value_type = Option<String>)]
    pub incus_address: Option<std::net::Ipv4Addr>,
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

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExternalNetwork {
    pub pool_uuid: uuid::Uuid,
    pub parent: String,
    pub vlan: Option<u16>,
    pub mtu: Option<u32>,
    pub mode: String,
    pub gvrp: bool,
    #[schema(value_type = String)]
    pub address: std::net::Ipv4Addr,
    pub prefix: u8,
    #[schema(value_type = String)]
    pub gateway: std::net::Ipv4Addr,
    pub gateway_onlink: bool,
    #[schema(value_type = Vec<String>)]
    pub dns: Vec<std::net::Ipv4Addr>,
    pub mac: Option<String>,
}

impl ExternalNetwork {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!((1..=32).contains(&self.prefix), "invalid IPv4 prefix");
        anyhow::ensure!(
            usable_ipv4(self.address) && usable_ipv4(self.gateway) && self.address != self.gateway,
            "invalid external IPv4 address or gateway"
        );
        anyhow::ensure!(
            !self.parent.is_empty()
                && self.parent.len() <= 15
                && self.parent != "lo"
                && self
                    .parent
                    .bytes()
                    .all(|value| value.is_ascii_alphanumeric()
                        || matches!(value, b'.' | b'_' | b'-')),
            "invalid parent interface"
        );
        anyhow::ensure!(
            self.vlan.is_none_or(|vlan| (1..=4094).contains(&vlan)),
            "VLAN must be 1 through 4094"
        );
        anyhow::ensure!(
            self.mtu.is_none_or(|mtu| (576..=65535).contains(&mtu)),
            "invalid MTU"
        );
        anyhow::ensure!(
            matches!(self.mode.as_str(), "bridge" | "private" | "vepa"),
            "invalid macvlan mode"
        );
        anyhow::ensure!(
            !self.dns.is_empty()
                && self.dns.len() <= 4
                && self.dns.iter().copied().all(usable_ipv4),
            "one through four IPv4 DNS servers are required"
        );
        if let Some(mac) = &self.mac {
            validate_mac(mac)?;
        }
        let mask = if self.prefix == 0 {
            0
        } else {
            u32::MAX << (32 - self.prefix)
        };
        let address = u32::from(self.address);
        if self.prefix <= 30 {
            anyhow::ensure!(
                address & !mask != 0 && address & !mask != !mask,
                "IPv4 address is a subnet or broadcast address"
            );
        }
        let gateway = u32::from(self.gateway);
        if self.prefix <= 30 && gateway & mask == address & mask {
            anyhow::ensure!(
                gateway & !mask != 0 && gateway & !mask != !mask,
                "gateway is a subnet or broadcast address"
            );
        }
        anyhow::ensure!(
            self.gateway_onlink || u32::from(self.gateway) & mask == address & mask,
            "gateway outside the subnet requires on-link routing"
        );
        Ok(())
    }
}

fn usable_ipv4(address: std::net::Ipv4Addr) -> bool {
    address.octets()[0] != 0
        && address.octets()[0] < 224
        && !address.is_unspecified()
        && !address.is_loopback()
        && !address.is_multicast()
        && !address.is_broadcast()
}

pub fn validate_mac(mac: &str) -> anyhow::Result<()> {
    let bytes = mac
        .split(':')
        .map(|part| {
            anyhow::ensure!(
                part.len() == 2 && part.bytes().all(|byte| byte.is_ascii_hexdigit()),
                "invalid MAC address"
            );
            Ok(u8::from_str_radix(part, 16)?)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    anyhow::ensure!(
        bytes.len() == 6
            && bytes.first().is_some_and(|byte| byte & 1 == 0)
            && bytes.iter().any(|value| *value != 0),
        "MAC must be a non-zero unicast address"
    );
    Ok(())
}

tokio::task_local! {
    pub static CREATING_NETWORK: Option<crate::pools::NetworkRequest>;
}
