use crate::model::{ExternalNetwork, validate_mac};
use anyhow::{Context, ensure};
use axum::{extract::Path, http::StatusCode};
use serde::{Deserialize, Serialize};
use shared::{
    GetState, State,
    models::{ByUuid, node::Node, user::GetPermissionManager},
    response::{ApiResponse, ApiResponseResult},
};
use sqlx::Row;
use std::net::Ipv4Addr;
use utoipa::ToSchema;

#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NetworkRequest {
    pub pool_uuid: uuid::Uuid,
    #[schema(value_type = Option<String>)]
    pub address: Option<Ipv4Addr>,
    pub mac: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PoolSettings {
    pub parent: String,
    pub vlan: Option<u16>,
    pub mtu: Option<u32>,
    pub mode: String,
    pub gvrp: bool,
    pub subnet: String,
    #[schema(value_type = String)]
    pub start: Ipv4Addr,
    #[schema(value_type = String)]
    pub end: Ipv4Addr,
    #[schema(value_type = String)]
    pub gateway: Ipv4Addr,
    pub gateway_onlink: bool,
    #[schema(value_type = Vec<String>)]
    pub dns: Vec<Ipv4Addr>,
}

impl PoolSettings {
    fn prefix(&self) -> anyhow::Result<u8> {
        let (address, prefix) = self
            .subnet
            .split_once('/')
            .context("subnet requires IPv4 CIDR")?;
        let address: Ipv4Addr = address.parse()?;
        let prefix: u8 = prefix.parse()?;
        ensure!(prefix <= 32, "invalid IPv4 prefix");
        let mask = if prefix == 0 {
            0
        } else {
            u32::MAX << (32 - prefix)
        };
        ensure!(
            u32::from(address) & !mask == 0,
            "subnet must use the network address"
        );
        ensure!(
            u32::from(self.start) & mask == u32::from(address)
                && u32::from(self.end) & mask == u32::from(address),
            "address range is outside the subnet"
        );
        Ok(prefix)
    }
    fn network(
        &self,
        pool_uuid: uuid::Uuid,
        address: Ipv4Addr,
        mac: Option<String>,
    ) -> anyhow::Result<ExternalNetwork> {
        let network = ExternalNetwork {
            pool_uuid,
            parent: self.parent.clone(),
            vlan: self.vlan,
            mtu: self.mtu,
            mode: self.mode.clone(),
            gvrp: self.gvrp,
            address,
            prefix: self.prefix()?,
            gateway: self.gateway,
            gateway_onlink: self.gateway_onlink,
            dns: self.dns.clone(),
            mac,
        };
        network.validate()?;
        Ok(network)
    }
    fn validate(&self) -> anyhow::Result<()> {
        ensure!(
            u32::from(self.start) <= u32::from(self.end)
                && u32::from(self.end) - u32::from(self.start) < 65536,
            "pool must contain one through 65536 addresses"
        );
        self.network(uuid::Uuid::nil(), self.start, None)?;
        self.network(uuid::Uuid::nil(), self.end, None)?;
        ensure!(
            self.gateway < self.start || self.gateway > self.end,
            "gateway cannot be in the assignable address range"
        );
        Ok(())
    }
}

#[derive(Serialize, ToSchema)]
pub struct Pool {
    pub uuid: uuid::Uuid,
    pub name: String,
    pub config: PoolSettings,
    pub total: i64,
    pub available: i64,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct Interface {
    pub name: String,
    pub mtu: u32,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub struct Lease {
    pub server_uuid: uuid::Uuid,
    #[schema(value_type = String)]
    pub address: Ipv4Addr,
}
#[derive(Deserialize, Serialize, ToSchema)]
pub struct Inventory {
    pub interfaces: Vec<Interface>,
    pub leases: Vec<Lease>,
}

async fn node(state: &State, uuid: uuid::Uuid) -> anyhow::Result<Node> {
    Node::by_uuid_optional(&state.database, uuid)
        .await?
        .context("node not found")
}

async fn inventory(state: &State, node: &Node) -> anyhow::Result<Inventory> {
    Ok(node
        .api_client(&state.database)
        .await?
        .request_raw(reqwest::Method::GET, "/api/system/incus/network")
        .header("Accept", "application/json")
        .send()
        .await?
        .error_for_status()
        .context("update Wings to enable direct Incus networking")?
        .json()
        .await?)
}

pub async fn reserve(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    node: uuid::Uuid,
    request: &NetworkRequest,
) -> anyhow::Result<ExternalNetwork> {
    if let Some(mac) = &request.mac {
        validate_mac(mac)
            .map_err(|error| shared::response::DisplayError::new(error.to_string()))?;
    }
    let config: serde_json::Value = sqlx::query_scalar(
        "SELECT config FROM xyz_luxxy_incus_ip_pools WHERE uuid = $1 AND node_uuid = $2 FOR SHARE",
    )
    .bind(request.pool_uuid)
    .bind(node)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| shared::response::DisplayError::new("IP pool is not on this node"))?;
    let address: String = sqlx::query_scalar("SELECT host(address) FROM xyz_luxxy_incus_ip_addresses WHERE pool_uuid = $1 AND server_uuid IS NULL AND ($2::inet IS NULL OR address = $2::inet) ORDER BY address FOR UPDATE SKIP LOCKED LIMIT 1")
        .bind(request.pool_uuid).bind(request.address.map(|ip| ip.to_string())).fetch_optional(&mut **transaction).await?.ok_or_else(|| shared::response::DisplayError::new("selected IP is unavailable or the pool is exhausted").with_status(StatusCode::CONFLICT))?;
    serde_json::from_value::<PoolSettings>(config)?.network(
        request.pool_uuid,
        address.parse()?,
        request.mac.clone(),
    )
}

pub async fn bind(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    server: uuid::Uuid,
    network: &ExternalNetwork,
) -> anyhow::Result<()> {
    let result = sqlx::query("UPDATE xyz_luxxy_incus_ip_addresses SET server_uuid = $1, mac = $4 WHERE pool_uuid = $2 AND address = $3::inet AND server_uuid IS NULL")
        .bind(server).bind(network.pool_uuid).bind(network.address.to_string())
        .bind(network.mac.as_ref().map(|mac| mac.to_ascii_lowercase())).execute(&mut **transaction).await
        .map_err(|error| if error.as_database_error().is_some_and(|error| error.is_unique_violation()) {
            anyhow::Error::from(shared::response::DisplayError::new("MAC address is already reserved on this node").with_status(StatusCode::CONFLICT))
        } else { error.into() })?;
    ensure!(
        result.rows_affected() == 1,
        "IP reservation changed before server creation"
    );
    Ok(())
}

pub mod list {
    use super::*;
    #[utoipa::path(get, path = "/incus/nodes/{node}/ip-pools", responses((status = OK, body = Vec<Pool>)))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        Path(uuid): Path<uuid::Uuid>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("nodes.read")?;
        let _ = node(&state, uuid).await?;
        let rows = sqlx::query("SELECT p.uuid, p.name, p.config, count(a.address)::bigint AS total, count(a.address) FILTER (WHERE a.server_uuid IS NULL)::bigint AS available FROM xyz_luxxy_incus_ip_pools p LEFT JOIN xyz_luxxy_incus_ip_addresses a ON a.pool_uuid = p.uuid WHERE p.node_uuid = $1 GROUP BY p.uuid ORDER BY p.name")
            .bind(uuid).fetch_all(state.database.write()).await?;
        let mut pools = Vec::new();
        for row in rows {
            pools.push(Pool {
                uuid: row.try_get("uuid")?,
                name: row.try_get("name")?,
                config: serde_json::from_value(row.try_get("config")?)?,
                total: row.try_get("total")?,
                available: row.try_get("available")?,
            });
        }
        ApiResponse::new_serialized(pools).ok()
    }
}

pub mod network {
    use super::*;
    #[utoipa::path(get, path = "/incus/nodes/{node}/network", responses((status = OK, body = Inventory)))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        Path(uuid): Path<uuid::Uuid>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("nodes.read")?;
        ApiResponse::new_serialized(inventory(&state, &node(&state, uuid).await?).await?).ok()
    }
}

pub mod create {
    use super::*;
    #[derive(Deserialize, ToSchema)]
    pub struct Payload {
        pub name: String,
        pub config: PoolSettings,
    }
    #[utoipa::path(post, path = "/incus/nodes/{node}/ip-pools", request_body = Payload, responses((status = OK)))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        Path(node_uuid): Path<uuid::Uuid>,
        shared::Payload(data): shared::Payload<Payload>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("nodes.allocations")?;
        if let Err(error) = data.config.validate() {
            return ApiResponse::error(error.to_string())
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }
        if data.name.trim().is_empty() || data.name.len() > 255 {
            return ApiResponse::error("pool name must contain 1 through 255 characters")
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }
        let node = node(&state, node_uuid).await?;
        let inventory = inventory(&state, &node).await?;
        if !inventory
            .interfaces
            .iter()
            .any(|interface| interface.name == data.config.parent)
        {
            return ApiResponse::error("parent interface does not exist on this node")
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }
        let uuid = uuid::Uuid::new_v4();
        let mut transaction = state.database.write().begin().await?;
        sqlx::query("INSERT INTO xyz_luxxy_incus_ip_pools (uuid, node_uuid, name, config) VALUES ($1, $2, $3, $4)")
            .bind(uuid).bind(node_uuid).bind(&data.name).bind(serde_json::to_value(&data.config)?).execute(&mut *transaction).await?;
        let addresses: Vec<String> = (u32::from(data.config.start)..=u32::from(data.config.end))
            .map(|ip| Ipv4Addr::from(ip).to_string())
            .collect();
        let result = sqlx::query("INSERT INTO xyz_luxxy_incus_ip_addresses (pool_uuid, node_uuid, address) SELECT $1, $2, address::inet FROM unnest($3::text[]) AS address")
            .bind(uuid).bind(node_uuid).bind(addresses).execute(&mut *transaction).await;
        if let Err(error) = result {
            if error
                .as_database_error()
                .is_some_and(|error| error.is_unique_violation())
            {
                return ApiResponse::error("address range overlaps an existing pool on this node")
                    .with_status(StatusCode::CONFLICT)
                    .ok();
            }
            return Err(error.into());
        }
        transaction.commit().await?;
        ApiResponse::new_serialized(serde_json::json!({"uuid":uuid})).ok()
    }
}

pub mod delete {
    use super::*;
    #[utoipa::path(delete, path = "/incus/nodes/{node}/ip-pools/{pool}", responses((status = OK), (status = CONFLICT)))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        Path((node, pool)): Path<(uuid::Uuid, uuid::Uuid)>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("nodes.allocations")?;
        let mut transaction = state.database.write().begin().await?;
        let exists: Option<uuid::Uuid> = sqlx::query_scalar("SELECT uuid FROM xyz_luxxy_incus_ip_pools WHERE uuid = $1 AND node_uuid = $2 FOR UPDATE").bind(pool).bind(node).fetch_optional(&mut *transaction).await?;
        if exists.is_none() {
            return ApiResponse::error("pool not found")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        }
        let occupied: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM xyz_luxxy_incus_ip_addresses WHERE pool_uuid = $1 AND server_uuid IS NOT NULL)").bind(pool).fetch_one(&mut *transaction).await?;
        if occupied {
            return ApiResponse::error(
                "pool has reserved IPs; delete their instances and reconcile first",
            )
            .with_status(StatusCode::CONFLICT)
            .ok();
        }
        sqlx::query("DELETE FROM xyz_luxxy_incus_ip_pools WHERE uuid = $1")
            .bind(pool)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        ApiResponse::new_serialized(serde_json::json!({})).ok()
    }
}

pub mod reconcile {
    use super::*;
    #[utoipa::path(post, path = "/incus/nodes/{node}/ip-pools/reconcile", responses((status = OK)))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        Path(uuid): Path<uuid::Uuid>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("nodes.allocations")?;
        let inventory = inventory(&state, &node(&state, uuid).await?).await?;
        let servers: Vec<uuid::Uuid> = inventory
            .leases
            .iter()
            .map(|lease| lease.server_uuid)
            .collect();
        let addresses: Vec<String> = inventory
            .leases
            .iter()
            .map(|lease| lease.address.to_string())
            .collect();
        let result = sqlx::query("UPDATE xyz_luxxy_incus_ip_addresses AS a SET server_uuid = NULL, mac = NULL WHERE a.node_uuid = $1 AND a.server_uuid IS NOT NULL AND NOT EXISTS (SELECT 1 FROM servers WHERE uuid = a.server_uuid) AND NOT (a.server_uuid = ANY($2::uuid[])) AND NOT (host(a.address) = ANY($3::text[]))")
            .bind(uuid).bind(servers).bind(addresses).execute(state.database.write()).await?;
        ApiResponse::new_serialized(serde_json::json!({"released":result.rows_affected()})).ok()
    }
}

pub mod addresses {
    use super::*;
    use axum::extract::Query;
    #[derive(Deserialize, ToSchema)]
    pub struct Pagination {
        #[serde(default)]
        pub offset: u32,
    }
    #[utoipa::path(get, path = "/incus/nodes/{node}/ip-pools/{pool}/addresses", responses((status = OK)))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        Path((node, pool)): Path<(uuid::Uuid, uuid::Uuid)>,
        Query(page): Query<Pagination>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("nodes.read")?;
        let rows = sqlx::query("SELECT host(a.address) AS address, a.server_uuid, s.name AS server_name FROM xyz_luxxy_incus_ip_addresses a LEFT JOIN servers s ON s.uuid = a.server_uuid WHERE a.node_uuid = $1 AND a.pool_uuid = $2 ORDER BY a.address LIMIT 100 OFFSET $3")
            .bind(node).bind(pool).bind(i64::from(page.offset)).fetch_all(state.database.write()).await?;
        let mut addresses = Vec::new();
        for row in rows {
            addresses.push(serde_json::json!({"address":row.try_get::<String, _>("address")?, "server_uuid":row.try_get::<Option<uuid::Uuid>, _>("server_uuid")?, "server_name":row.try_get::<Option<String>, _>("server_name")?}));
        }
        ApiResponse::new_serialized(addresses).ok()
    }
}
