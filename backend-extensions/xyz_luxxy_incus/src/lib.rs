use shared::{
    State,
    extensions::{Extension, ExtensionRouteBuilder},
    models::{
        BaseModel, CreatableModel, ListenerPriority, UpdatableModel,
        server::{AdminApiServer, ApiServer, Server},
    },
};
use utoipa_axum::routes;

mod model;
mod routes;

#[derive(Default)]
pub struct ExtensionStruct;

#[async_trait::async_trait]
impl Extension for ExtensionStruct {
    async fn initialize(&mut self, _state: State) {
        Server::register_model_extension(model::ServerExtension);
        ApiServer::extend(
            |server, _state| {
                Box::pin(
                    async move { Ok(server.parse_model_extension::<model::ServerExtension>()?) },
                )
            },
            |_server, data, _state| model::ApiMetadata {
                incus_instance: data.instance.map(model::NativeInstance::without_config),
            },
        );
        AdminApiServer::extend(
            |server, _state| {
                Box::pin(
                    async move { Ok(server.parse_model_extension::<model::ServerExtension>()?) },
                )
            },
            |_server, data, _state| model::ApiMetadata {
                incus_instance: data.instance,
            },
        );
        Server::register_create_handler(
            ListenerPriority::Normal,
            |options, builder, _state, _transaction| {
                Box::pin(async move {
                    if let Ok(instance) = model::CREATING_INSTANCE.try_with(Clone::clone) {
                        instance.validate()?;
                        model::require(
                            options.egg_uuid == model::OS_EGG,
                            "native instances require the OS template",
                        )?;
                        builder.set("xyz_luxxy_incus_instance", serde_json::to_value(instance)?);
                    } else {
                        model::require(
                            options.egg_uuid != model::OS_EGG,
                            "create native instances through the Incus extension",
                        )?;
                    }
                    Ok(())
                })
            },
        );
        Server::register_update_handler(
            ListenerPriority::Normal,
            |server, options, builder, _state, _transaction| {
                Box::pin(async move {
                    let data = server.parse_model_extension::<model::ServerExtension>()?;
                    if let Some(instance) = data.instance {
                        if let Ok(next) = model::UPDATING_INSTANCE.try_with(Clone::clone) {
                            next.validate()?;
                            model::require(
                                next.kind == instance.kind && next.image == instance.image,
                                "changing instance type or image requires a new server",
                            )?;
                            builder.set(
                                "xyz_luxxy_incus_instance",
                                Some(serde_json::to_value(next)?),
                            );
                        }
                        model::require(
                            options.egg_uuid.is_none_or(|egg| egg == model::OS_EGG),
                            "native instances require the OS template",
                        )?;
                        model::require(
                            options
                                .image
                                .as_ref()
                                .is_none_or(|image| image.as_str() == instance.image),
                            "changing an OS image requires an explicit reinstall",
                        )?;
                        if let Some(limits) = &options.limits {
                            model::require(
                                limits.disk > 0,
                                "native instances require a positive disk size",
                            )?;
                            model::require(
                                instance.kind != model::InstanceKind::VirtualMachine
                                    || limits.memory >= 256,
                                "VMs require at least 256 MiB of memory",
                            )?;
                        }
                    } else {
                        model::require(
                            options.egg_uuid != Some(model::OS_EGG),
                            "changing instance type requires a new server",
                        )?;
                    }
                    Ok(())
                })
            },
        );
    }

    async fn initialize_router(
        &mut self,
        _state: State,
        builder: ExtensionRouteBuilder,
    ) -> ExtensionRouteBuilder {
        builder
            .add_admin_api_router(|router| {
                router
                    .routes(routes!(routes::create::route))
                    .routes(routes!(routes::runtime::route))
                    .routes(routes!(routes::images::route))
                    .routes(routes!(routes::admin_metadata::route))
                    .routes(routes!(routes::update::route))
            })
            .add_client_server_api_router(|router| {
                router.routes(routes!(routes::client_metadata::route))
            })
            .add_remote_api_router(|router| router.routes(routes!(routes::remote_metadata::route)))
    }
}
