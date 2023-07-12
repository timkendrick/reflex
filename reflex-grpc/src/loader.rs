// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
use std::{marker::PhantomData, path::Path};

use derivative::Derivative;
use reflex::core::{
    create_record, Builtin, Expression, ExpressionFactory, HeapAllocator, ModuleLoader,
};
use reflex_stdlib::{CollectList, Contains, Effect, Get, If, ResolveDeep};

use crate::{
    actor::EFFECT_TYPE_GRPC,
    proto::google::protobuf::ServiceDescriptorProto,
    utils::{get_proto_checksum, load_proto_descriptor, ProtoId},
};

pub trait GrpcLoaderBuiltin:
    Builtin
    + From<CollectList>
    + From<Contains>
    + From<Effect>
    + From<Get>
    + From<If>
    + From<ResolveDeep>
{
}
impl<T> GrpcLoaderBuiltin for T where
    T: Builtin
        + From<CollectList>
        + From<Contains>
        + From<Effect>
        + From<Get>
        + From<If>
        + From<ResolveDeep>
{
}

pub fn create_grpc_loader<
    T: Expression,
    TFactory: ExpressionFactory<T> + Clone + 'static,
    TAllocator: HeapAllocator<T> + Clone + 'static,
>(
    factory: &TFactory,
    allocator: &TAllocator,
) -> GrpcModuleLoader<T, TFactory, TAllocator>
where
    T::Builtin: GrpcLoaderBuiltin,
{
    GrpcModuleLoader::new(factory.clone(), allocator.clone())
}

#[derive(Derivative)]
#[derivative(Clone(bound = "TFactory: Clone, TAllocator: Clone"))]
pub struct GrpcModuleLoader<
    T: Expression,
    TFactory: ExpressionFactory<T>,
    TAllocator: HeapAllocator<T>,
> {
    factory: TFactory,
    allocator: TAllocator,
    _expression: PhantomData<T>,
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>>
    GrpcModuleLoader<T, TFactory, TAllocator>
{
    fn new(factory: TFactory, allocator: TAllocator) -> Self {
        Self {
            factory,
            allocator,
            _expression: PhantomData,
        }
    }
}

impl<T: Expression, TFactory: ExpressionFactory<T>, TAllocator: HeapAllocator<T>> ModuleLoader
    for GrpcModuleLoader<T, TFactory, TAllocator>
where
    T::Builtin: GrpcLoaderBuiltin,
{
    type Output = T;
    fn load(&self, import_path: &str, current_path: &Path) -> Option<Result<Self::Output, String>> {
        if !import_path.ends_with(".proto.bin") {
            return None;
        }
        let proto_path = current_path
            .parent()
            .map(|parent| parent.join(import_path))
            .unwrap_or_else(|| Path::new(import_path).to_path_buf());
        Some(match std::fs::read(&proto_path) {
            Err(err) => Err(format!("Failed to load protobuf schema: {}", err)),
            Ok(bytes) => load_proto_descriptor(bytes.as_slice())
                .map_err(|err| format!("{}", err))
                .and_then(|proto| {
                    let proto_id = get_proto_checksum(&proto);
                    let services = proto.file.iter().flat_map(|file| file.service.iter());
                    Ok(create_grpc_exports(
                        proto_id,
                        services,
                        &self.factory,
                        &self.allocator,
                    ))
                }),
        })
    }
}

fn create_grpc_exports<'a, T: Expression>(
    proto_id: ProtoId,
    services: impl IntoIterator<Item = &'a ServiceDescriptorProto>,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> T
where
    T::Builtin: GrpcLoaderBuiltin,
{
    // TODO: generate protobuf message constructors
    // TODO: generate protobuf enum constructors
    let services = services
        .into_iter()
        .map(|service| create_service_constructor(service, proto_id, factory, allocator));
    let exports = services;
    create_record(
        exports.map(|(key, value)| {
            (
                factory.create_string_term(allocator.create_string(key)),
                value,
            )
        }),
        factory,
        allocator,
    )
}

fn create_service_constructor<'a, T: Expression>(
    service: &'a ServiceDescriptorProto,
    proto_id: ProtoId,
    factory: &impl ExpressionFactory<T>,
    allocator: &impl HeapAllocator<T>,
) -> (&'a str, T)
where
    T::Builtin: GrpcLoaderBuiltin,
{
    (
        service.name(),
        factory.create_lambda_term(
            1,
            create_record(
                service.method.iter().map(|method| {
                    (
                        factory.create_string_term(allocator.create_string(method.name())),
                        factory.create_lambda_term(
                            2,
                            factory.create_application_term(
                                factory.create_builtin_term(Effect),
                                allocator.create_triple(
                                    factory.create_string_term(
                                        allocator.create_string(EFFECT_TYPE_GRPC),
                                    ),
                                    factory.create_application_term(
                                        factory.create_builtin_term(CollectList),
                                        allocator.create_list([
                                            factory.create_symbol_term(proto_id.into()),
                                            factory.create_application_term(
                                                factory.create_builtin_term(Get),
                                                allocator.create_pair(
                                                    factory.create_variable_term(2),
                                                    factory.create_string_term(
                                                        allocator
                                                            .create_string(String::from("url")),
                                                    ),
                                                ),
                                            ),
                                            factory.create_string_term(
                                                allocator
                                                    .create_string(String::from(service.name())),
                                            ),
                                            factory.create_string_term(
                                                allocator
                                                    .create_string(String::from(method.name())),
                                            ),
                                            factory.create_application_term(
                                                factory.create_builtin_term(ResolveDeep),
                                                allocator.create_unit_list(
                                                    factory.create_variable_term(1),
                                                ),
                                            ),
                                            factory.create_nil_term(),
                                        ]),
                                    ),
                                    factory.create_application_term(
                                        factory.create_builtin_term(Get),
                                        allocator.create_pair(
                                            factory.create_variable_term(0),
                                            factory.create_string_term(
                                                allocator.create_string(String::from("token")),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    )
                }),
                factory,
                allocator,
            ),
        ),
    )
}

#[cfg(test)]
mod tests {
    use reflex_protobuf::{
        load_proto_library,
        well_known_types::{
            field_descriptor_proto::{Label, Type},
            DescriptorProto, FieldDescriptorProto, FileDescriptorProto, MethodDescriptorProto,
            ServiceDescriptorProto,
        },
    };

    #[test]
    fn deserialize_proto() {
        let protos = load_proto_library(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/grpc/mocks.proto.bin"
        )))
        .unwrap();
        let service_proto = protos
            .as_inner()
            .file_descriptor_protos()
            .find(|proto| proto.name == Some(String::from("hello_world_service.proto")));
        assert_eq!(
            service_proto,
            Some(&FileDescriptorProto {
                syntax: Some(String::from("proto3")),
                name: Some(String::from("hello_world_service.proto")),
                package: Some(String::from("hello_world_service")),
                dependency: Default::default(),
                public_dependency: Default::default(),
                weak_dependency: Default::default(),
                service: vec![ServiceDescriptorProto {
                    name: Some(String::from("HelloWorldService")),
                    method: vec![MethodDescriptorProto {
                        name: Some(String::from("Greet")),
                        input_type: Some(String::from(".hello_world_service.GreetRequest")),
                        output_type: Some(String::from(".hello_world_service.GreetResponse")),
                        options: Default::default(),
                        client_streaming: None,
                        server_streaming: None,
                    }],
                    options: Default::default(),
                }],
                message_type: vec![
                    DescriptorProto {
                        name: Some(String::from("GreetRequest")),
                        field: vec![FieldDescriptorProto {
                            name: Some(String::from("user")),
                            number: Some(1),
                            label: Some(Label::Optional.into()),
                            r#type: Some(Type::String.into()),
                            type_name: None,
                            extendee: Default::default(),
                            default_value: Default::default(),
                            oneof_index: Default::default(),
                            json_name: Some(String::from("user")),
                            options: Default::default(),
                            proto3_optional: Default::default(),
                        }],
                        extension: Default::default(),
                        nested_type: Default::default(),
                        enum_type: Default::default(),
                        extension_range: Default::default(),
                        oneof_decl: Default::default(),
                        options: Default::default(),
                        reserved_range: Default::default(),
                        reserved_name: Default::default(),
                    },
                    DescriptorProto {
                        name: Some(String::from("GreetResponse")),
                        field: vec![FieldDescriptorProto {
                            name: Some(String::from("message")),
                            number: Some(1),
                            label: Some(Label::Optional.into()),
                            r#type: Some(Type::String.into()),
                            type_name: None,
                            extendee: Default::default(),
                            default_value: Default::default(),
                            oneof_index: Default::default(),
                            json_name: Some(String::from("message")),
                            options: Default::default(),
                            proto3_optional: Default::default(),
                        }],
                        extension: Default::default(),
                        nested_type: Default::default(),
                        enum_type: Default::default(),
                        extension_range: Default::default(),
                        oneof_decl: Default::default(),
                        options: Default::default(),
                        reserved_range: Default::default(),
                        reserved_name: Default::default(),
                    },
                ],
                enum_type: Default::default(),
                extension: Default::default(),
                options: Default::default(),
                source_code_info: Default::default(),
            })
        );
    }
}
