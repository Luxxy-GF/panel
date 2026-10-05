use crate::model::{InstanceKind, NativeInstance, OS_EGG, ServerExtension};
use axum::{extract::Path, http::StatusCode};
use serde::{Deserialize, Serialize};
use shared::{
    GetState, State,
    models::{
        BaseModel, ByUuid, CreatableModel, IntoAdminApiObject,
        node::{GetNode, Node},
        server::{CreateServerOptions, GetServer, Server},
        user::GetPermissionManager,
    },
    response::{ApiResponse, ApiResponseResult},
};
use sqlx::Row;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct RuntimeCapabilities {
    pub backend: String,
    pub system_containers: bool,
    pub virtual_machines: bool,
    pub image_server: Option<String>,
    #[serde(default)]
    pub panel_extension: bool,
}

#[derive(Deserialize)]
struct SystemInfo {
    runtime: Option<RuntimeCapabilities>,
}

#[derive(Deserialize)]
struct NodeConfiguration {
    runtime: RuntimeConfiguration,
}

#[derive(Deserialize)]
struct RuntimeConfiguration {
    incus: IncusConfiguration,
}

#[derive(Deserialize)]
struct IncusConfiguration {
    #[serde(default)]
    panel_extension: bool,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct NativeImage {
    pub alias: String,
    pub label: String,
    pub kind: InstanceKind,
}

async fn node_runtime(state: &State, node: &Node) -> anyhow::Result<Option<RuntimeCapabilities>> {
    let client = node.api_client(&state.database).await?;
    let mut runtime = client
        .request_raw(reqwest::Method::GET, "/api/system")
        .header("Accept", "application/json")
        .send()
        .await?
        .error_for_status()?
        .json::<SystemInfo>()
        .await?
        .runtime;
    if let Some(runtime) = &mut runtime {
        runtime.panel_extension = false;
        if runtime.backend == "incus" {
            runtime.panel_extension = client
                .request_raw(reqwest::Method::GET, "/api/system/config")
                .header("Accept", "application/json")
                .send()
                .await?
                .error_for_status()?
                .json::<NodeConfiguration>()
                .await?
                .runtime
                .incus
                .panel_extension;
        }
    }
    Ok(runtime)
}

async fn node_images(state: &State, node: &Node) -> anyhow::Result<Vec<NativeImage>> {
    Ok(node
        .api_client(&state.database)
        .await?
        .request_raw(reqwest::Method::GET, "/api/system/images")
        .header("Accept", "application/json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

pub async fn validate_node(
    state: &State,
    uuid: uuid::Uuid,
    instance: &NativeInstance,
) -> anyhow::Result<()> {
    let node = Node::by_uuid_optional(&state.database, uuid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("node not found"))?;
    let runtime = node_runtime(state, &node)
        .await?
        .ok_or_else(|| anyhow::anyhow!("node does not report native instance support"))?;
    anyhow::ensure!(
        runtime.backend == "incus" && runtime.system_containers,
        "native instances require an Incus node"
    );
    anyhow::ensure!(
        runtime.panel_extension,
        "enable runtime.incus.panel_extension on Wings and restart the daemon"
    );
    anyhow::ensure!(
        instance.kind != InstanceKind::VirtualMachine || runtime.virtual_machines,
        "node does not support virtual machines"
    );
    Ok(())
}

#[derive(Serialize, ToSchema)]
struct MetadataResponse {
    instance: Option<NativeInstance>,
}

pub mod create {
    use super::*;
    use garde::Validate;
    use shared::models::admin_activity::GetAdminActivityLogger;

    #[derive(Deserialize, ToSchema, Validate)]
    pub struct Payload {
        #[garde(skip)]
        pub instance: NativeInstance,
        #[serde(flatten)]
        #[garde(dive)]
        pub options: CreateServerOptions,
    }

    #[derive(Serialize, ToSchema)]
    struct Response {
        server: serde_json::Value,
    }

    #[utoipa::path(post, path = "/incus/servers", request_body = Payload,
        responses((status = OK, body = Response)))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        activity: GetAdminActivityLogger,
        shared::Payload(mut data): shared::Payload<Payload>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("servers.create")?;
        if let Err(error) = data.instance.validate() {
            return ApiResponse::error(error.to_string())
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }
        if let Err(errors) = shared::utils::validate_data(&data) {
            return ApiResponse::new_serialized(shared::ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }
        if data.options.limits.disk <= 0
            || (data.instance.kind == InstanceKind::VirtualMachine
                && data.options.limits.memory < 256)
        {
            return ApiResponse::error("native instances require a positive disk size; VMs require at least 256 MiB of memory").with_status(StatusCode::BAD_REQUEST).ok();
        }
        if let Err(error) = validate_node(&state, data.options.node_uuid, &data.instance).await {
            return ApiResponse::error(error.to_string())
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }
        let node = Node::by_uuid_optional(&state.database, data.options.node_uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("node not found"))?;
        if !node_images(&state, &node)
            .await?
            .iter()
            .any(|image| image.kind == data.instance.kind && image.alias == data.instance.image)
        {
            return ApiResponse::error("selected image is not available for this instance type")
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }
        data.options.egg_uuid = OS_EGG;
        data.options.image = data.instance.image.clone().into();
        data.options.startup = "/sbin/init".into();
        data.options.skip_installer = true;
        data.options.variables.clear();
        data.options.hugepages_passthrough_enabled = false;
        data.options.kvm_passthrough_enabled = false;
        let instance = data.instance;
        let server = crate::model::CREATING_INSTANCE
            .scope(instance.clone(), Server::create(&state, data.options))
            .await?;
        activity.log("server:create", serde_json::json!({"uuid":server.uuid,"extension":"xyz.luxxy.incus","instance_kind":instance.kind,"image":instance.image})).await;
        let value = serde_json::to_value(
            server
                .into_admin_api_object(&state, &state.storage.retrieve_urls().await?)
                .await?,
        )?;
        ApiResponse::new_serialized(Response { server: value }).ok()
    }
}

pub mod update {
    use super::*;
    use garde::Validate;
    use shared::models::{
        UpdatableModel, admin_activity::GetAdminActivityLogger, server::UpdateServerOptions,
    };

    #[derive(Deserialize, ToSchema, Validate)]
    pub struct Payload {
        #[garde(skip)]
        pub instance: NativeInstance,
        #[serde(flatten)]
        #[garde(dive)]
        pub options: UpdateServerOptions,
    }

    #[derive(Deserialize)]
    struct ServerStatus {
        state: String,
    }

    #[utoipa::path(patch, path = "/incus/servers/{server}", request_body = Payload,
        responses((status = OK), (status = BAD_REQUEST), (status = CONFLICT)))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        activity: GetAdminActivityLogger,
        Path(uuid): Path<uuid::Uuid>,
        shared::Payload(data): shared::Payload<Payload>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("servers.read")?;
        permissions.has_admin_permission("servers.update")?;
        if let Err(error) = data.instance.validate() {
            return ApiResponse::error(error.to_string())
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }
        if let Err(errors) = shared::utils::validate_data(&data) {
            return ApiResponse::new_serialized(shared::ApiError::new_strings_value(errors))
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }
        let mut server = Server::by_uuid_optional(&state.database, uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("server not found"))?;
        let Some(existing) = server.parse_model_extension::<ServerExtension>()?.instance else {
            return ApiResponse::error("server is not a native Incus instance")
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        };
        if existing.kind != data.instance.kind || existing.image != data.instance.image {
            return ApiResponse::error("changing instance type or image requires a new server")
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }
        if existing.config != data.instance.config {
            let node = server.node.fetch_cached(&state.database).await?;
            let status = node
                .api_client(&state.database)
                .await?
                .request_raw(reqwest::Method::GET, &format!("/api/servers/{uuid}"))
                .header("Accept", "application/json")
                .send()
                .await?
                .error_for_status()?
                .json::<ServerStatus>()
                .await?;
            if status.state != "offline" {
                return ApiResponse::error("stop the instance before changing Incus configuration")
                    .with_status(StatusCode::CONFLICT)
                    .ok();
            }
        }
        let instance = data.instance;
        crate::model::UPDATING_INSTANCE
            .scope(instance, server.update(&state, data.options))
            .await?;
        activity
            .log(
                "server:update",
                serde_json::json!({"uuid":uuid,"extension":"xyz.luxxy.incus"}),
            )
            .await;
        server.sync(&state.database).await?;
        ApiResponse::new_serialized(serde_json::json!({})).ok()
    }
}

pub mod runtime {
    use super::*;
    #[utoipa::path(get, path = "/incus/nodes/{node}/runtime", responses((status = OK, body = Option<RuntimeCapabilities>)))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        Path(uuid): Path<uuid::Uuid>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("nodes.read")?;
        let node = Node::by_uuid_optional(&state.database, uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("node not found"))?;
        ApiResponse::new_serialized(node_runtime(&state, &node).await?).ok()
    }
}

pub mod images {
    use super::*;
    #[utoipa::path(get, path = "/incus/nodes/{node}/images", responses((status = OK, body = Vec<NativeImage>)))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        Path(uuid): Path<uuid::Uuid>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("nodes.read")?;
        let node = Node::by_uuid_optional(&state.database, uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("node not found"))?;
        ApiResponse::new_serialized(node_images(&state, &node).await?).ok()
    }
}

pub mod admin_metadata {
    use super::*;
    #[utoipa::path(get, path = "/incus/servers/{server}", responses((status = OK, body = MetadataResponse)))]
    pub async fn route(
        state: GetState,
        permissions: GetPermissionManager,
        Path(uuid): Path<uuid::Uuid>,
    ) -> ApiResponseResult {
        permissions.has_admin_permission("servers.read")?;
        let server = Server::by_uuid_optional(&state.database, uuid)
            .await?
            .ok_or_else(|| anyhow::anyhow!("server not found"))?;
        ApiResponse::new_serialized(MetadataResponse {
            instance: server.parse_model_extension::<ServerExtension>()?.instance,
        })
        .ok()
    }
}

pub mod client_metadata {
    use super::*;
    #[utoipa::path(get, path = "/incus", responses((status = OK, body = MetadataResponse)))]
    pub async fn route(server: GetServer, permissions: GetPermissionManager) -> ApiResponseResult {
        permissions.has_server_permission("control.read-console")?;
        ApiResponse::new_serialized(MetadataResponse {
            instance: server
                .parse_model_extension::<ServerExtension>()?
                .instance
                .map(NativeInstance::without_config),
        })
        .ok()
    }
}

pub mod remote_metadata {
    use super::*;
    #[derive(Deserialize, ToSchema)]
    pub struct Payload {
        uuids: Vec<uuid::Uuid>,
    }
    #[derive(Serialize, ToSchema)]
    struct Entry {
        uuid: uuid::Uuid,
        instance: Option<NativeInstance>,
    }
    #[derive(Serialize, ToSchema)]
    struct Response {
        version: u32,
        servers: Vec<Entry>,
    }
    #[utoipa::path(post, path = "/incus/servers", request_body = Payload, responses((status = OK, body = Response)))]
    pub async fn route(
        state: GetState,
        node: GetNode,
        shared::Payload(data): shared::Payload<Payload>,
    ) -> ApiResponseResult {
        let uuids: std::collections::BTreeSet<_> = data.uuids.into_iter().collect();
        if uuids.len() > 1000 {
            return ApiResponse::error("metadata batches are limited to 1000 servers")
                .with_status(StatusCode::BAD_REQUEST)
                .ok();
        }
        let rows = sqlx::query("SELECT uuid, xyz_luxxy_incus_instance FROM servers WHERE node_uuid = $1 AND uuid = ANY($2)")
            .bind(node.uuid).bind(uuids.iter().copied().collect::<Vec<_>>()).fetch_all(state.database.write()).await?;
        if rows.len() != uuids.len() {
            return ApiResponse::error("server not found on this node")
                .with_status(StatusCode::NOT_FOUND)
                .ok();
        }
        let mut servers = Vec::with_capacity(rows.len());
        for row in rows {
            let value: Option<serde_json::Value> = row.try_get("xyz_luxxy_incus_instance")?;
            servers.push(Entry {
                uuid: row.try_get("uuid")?,
                instance: value.map(serde_json::from_value).transpose()?,
            });
        }
        ApiResponse::new_serialized(Response {
            version: 1,
            servers,
        })
        .ok()
    }
}
