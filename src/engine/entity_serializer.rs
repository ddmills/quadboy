use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub use bevy_serializable_derive::SerializableComponent;

pub trait SerializableComponent:
    Component + Clone + Serialize + for<'de> Deserialize<'de> + 'static
{
    fn to_serializable(&self) -> Box<dyn SerializableValue>;
    fn from_serializable(value: &dyn SerializableValue, cmds: &mut EntityCommands);
    fn type_name() -> &'static str;
    fn type_id() -> TypeId {
        TypeId::of::<Self>()
    }
}

pub trait SerializableValue {
    fn as_any(&self) -> &dyn Any;
    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error>;
}

impl<T> SerializableValue for T
where
    T: Serialize + for<'de> Deserialize<'de> + Clone + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }
}

type SerializerFn = Box<dyn Fn(&World, Entity) -> Option<Box<dyn SerializableValue>> + Send + Sync>;
type DeserializerFn = Box<
    dyn Fn(&serde_json::Value) -> Result<Box<dyn SerializableValue>, serde_json::Error>
        + Send
        + Sync,
>;
type InserterFn = Box<dyn Fn(&dyn SerializableValue, &mut EntityCommands) + Send + Sync>;

#[derive(Resource, Default)]
pub struct SerializableComponentRegistry {
    serializers: HashMap<TypeId, SerializerFn>,
    deserializers: HashMap<String, DeserializerFn>,
    inserters: HashMap<String, InserterFn>,
    type_names: HashMap<TypeId, String>,
}

impl SerializableComponentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<T: SerializableComponent>(&mut self) {
        let type_id = T::type_id();
        let type_name = T::type_name().to_string();

        self.type_names.insert(type_id, type_name.clone());

        self.serializers.insert(
            type_id,
            Box::new(|world: &World, entity: Entity| {
                world
                    .get::<T>(entity)
                    .map(|component| component.to_serializable())
            }),
        );

        self.deserializers.insert(
            type_name.clone(),
            Box::new(|value: &serde_json::Value| -> Result<Box<dyn SerializableValue>, serde_json::Error> {
                let deserialized: T = serde_json::from_value(value.clone())?;
                Ok(Box::new(deserialized))
            })
        );

        self.inserters.insert(
            type_name,
            Box::new(|value: &dyn SerializableValue, cmds: &mut EntityCommands| {
                T::from_serializable(value, cmds);
            }),
        );
    }

    pub fn get_type_name(&self, type_id: &TypeId) -> Option<&String> {
        self.type_names.get(type_id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializedComponentData {
    pub type_name: String,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializedEntity {
    pub components: Vec<SerializedComponentData>,
}

// Trait to define what can be serialized
pub trait SerializableInput {
    fn get_entities(self, world: &World) -> Vec<Entity>;
}

impl SerializableInput for Vec<Entity> {
    fn get_entities(self, _world: &World) -> Vec<Entity> {
        self
    }
}

impl SerializableInput for &[Entity] {
    fn get_entities(self, _world: &World) -> Vec<Entity> {
        self.to_vec()
    }
}

impl SerializableInput for Entity {
    fn get_entities(self, _world: &World) -> Vec<Entity> {
        vec![self]
    }
}

impl<'w, 's> SerializableInput for Query<'w, 's, Entity> {
    fn get_entities(self, _world: &World) -> Vec<Entity> {
        self.iter().collect()
    }
}

// Trait to define what can be deserialized
pub trait DeserializableInput {
    fn get_entities(&self) -> Vec<&SerializedEntity>;
}

impl DeserializableInput for Vec<SerializedEntity> {
    fn get_entities(&self) -> Vec<&SerializedEntity> {
        self.iter().collect()
    }
}

impl DeserializableInput for &[SerializedEntity] {
    fn get_entities(&self) -> Vec<&SerializedEntity> {
        self.iter().collect()
    }
}

impl DeserializableInput for SerializedEntity {
    fn get_entities(&self) -> Vec<&SerializedEntity> {
        vec![&self]
    }
}

pub struct EntitySerializer;

impl EntitySerializer {
    pub fn serialize<T: SerializableInput>(
        input: T,
        world: &World,
        registry: &SerializableComponentRegistry,
    ) -> Vec<SerializedEntity> {
        let entities = input.get_entities(world);
        let mut serialized_entities = Vec::new();

        for entity in entities.iter() {
            let mut components = Vec::new();

            for (type_id, serializer) in &registry.serializers {
                if let Some(serializable_value) = serializer(world, *entity)
                    && let Some(type_name) = registry.get_type_name(type_id)
                {
                    match serializable_value.to_json() {
                        Ok(json_value) => {
                            components.push(SerializedComponentData {
                                type_name: type_name.clone(),
                                data: json_value,
                            });
                        }
                        Err(e) => {
                            eprintln!("Failed to serialize component {}: {}", type_name, e);
                        }
                    }
                }
            }

            if !components.is_empty() {
                serialized_entities.push(SerializedEntity { components });
            }
        }

        serialized_entities
    }

    pub fn deserialize_entity(
        world: &mut World,
        serialized_entity: SerializedEntity,
        // registry: &SerializableComponentRegistry,
    ) -> Entity {
        world
            .run_system_once_with(deser, serialized_entity)
            .unwrap()
    }

    pub fn deserialize<T: DeserializableInput>(world: &mut World, input: &T) -> Vec<Entity> {
        input
            .get_entities()
            .iter()
            .map(|e| EntitySerializer::deserialize_entity(world, e.to_owned().clone()))
            .collect()
    }
}

fn deser(
    In(serialized_entity): In<SerializedEntity>,
    mut cmds: Commands,
    registry: Res<SerializableComponentRegistry>,
) -> Entity {
    let mut e_cmds = cmds.spawn_empty();
    let entity = e_cmds.id();

    for component_data in serialized_entity.components.iter() {
        if let Some(deserializer) = registry.deserializers.get(&component_data.type_name) {
            match deserializer(&component_data.data) {
                Ok(serializable_value) => {
                    if let Some(inserter) = registry.inserters.get(&component_data.type_name) {
                        inserter(serializable_value.as_ref(), &mut e_cmds);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Failed to deserialize component {}: {}",
                        component_data.type_name, e
                    );
                }
            }
        }
    }

    entity
}

pub fn serialize(entity: Entity, world: &World) -> SerializedEntity {
    let registry = world
        .get_resource::<SerializableComponentRegistry>()
        .unwrap();
    let serialized = EntitySerializer::serialize(entity, world, registry);
    serialized.into_iter().next().unwrap()
}

pub fn deserialize_all(data: &Vec<SerializedEntity>, world: &mut World) -> Vec<Entity> {
    EntitySerializer::deserialize(world, data)
}

pub fn deserialize(data: SerializedEntity, world: &mut World) -> Entity {
    EntitySerializer::deserialize_entity(world, data)
}
