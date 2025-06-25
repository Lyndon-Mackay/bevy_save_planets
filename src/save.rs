use std::io::{Read, Write};

use bevy::{platform::collections::HashSet, prelude::*};

use bevy::ecs::entity::MapEntities;
use bevy::ecs::reflect::ReflectMapEntities;
use bevy_save::prelude::*;
use io_adapters::WriteExtension;
use serde::{Serialize, de::DeserializeSeed};

use crate::{MoonOf, OrbitingMoons, Planet, Star, StellarBodyOf};

pub struct PlanetSavePlugin;

impl Plugin for PlanetSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SavePlugins)
            .register_type::<PlanetPrefab>()
            .register_type::<StarPrefab>();
    }
}

pub struct RONFormat;

impl Format for RONFormat {
    fn extension() -> &'static str {
        ".ron"
    }

    fn serialize<W: Write, T: Serialize>(writer: W, value: &T) -> Result<(), Error> {
        let mut ser = ron::Serializer::new(
            writer.write_adapter(),
            Some(ron::ser::PrettyConfig::default()),
        )
        .map_err(Error::saving)?;

        value.serialize(&mut ser).map_err(Error::saving)
    }

    fn deserialize<R: Read, S: for<'de> DeserializeSeed<'de, Value = T>, T>(
        reader: R,
        seed: S,
    ) -> Result<T, Error> {
        ron::options::Options::default()
            .from_reader_seed(reader, seed)
            .map_err(Error::loading)
    }
}

pub struct PlanetPipeline;

impl Pipeline for PlanetPipeline {
    type Backend = DefaultBackend;

    type Format = RONFormat;

    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        "pronytic"
    }

    fn capture(&self, builder: SnapshotBuilder) -> Snapshot {
        let mut deny_list = HashSet::new();

        deny_list.insert(std::any::TypeId::of::<Time<Fixed>>());
        deny_list.insert(std::any::TypeId::of::<Time>());
        deny_list.insert(std::any::TypeId::of::<Time<Real>>());
        deny_list.insert(std::any::TypeId::of::<Time<Virtual>>());

        builder
            .filter(SceneFilter::Denylist(deny_list))
            .extract_all_resources()
            .allow_all()
            // .extract_entities_matching(|e| e.contains::<OrbitingMoons>())
            .extract_all_prefabs::<PlanetPrefab>()
            .extract_all_prefabs::<StarPrefab>()
            // .extract_entities_matching(|e| e.contains::<Save>())
            .build()
    }

    fn apply(&self, world: &mut World, snapshot: &Snapshot) -> Result<(), Error> {
        // let mut hash_map = EntityHashMap::new();
        snapshot
            .applier(world)
            // .entity_map(&mut hash_map)
            .despawn::<Or<(With<Planet>, With<Star>)>>()
            .prefab::<PlanetPrefab>()
            .prefab::<StarPrefab>()
            .apply()
    }
}

#[derive(Reflect, Debug)]
#[reflect(MapEntities)]
pub struct PlanetPrefab {
    transform: Transform,
    moon_of: Option<MoonOf>,
    star_of: Option<StellarBodyOf>,
}

impl MapEntities for PlanetPrefab {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {
        if let Some(moon_of) = &mut self.moon_of {
            moon_of.0 = entity_mapper.get_mapped(moon_of.0);
        }
        if let Some(star_of) = &mut self.star_of {
            star_of.0 = entity_mapper.get_mapped(star_of.0);
        }
    }
}

impl Prefab for PlanetPrefab {
    type Marker = Planet;

    fn spawn(self, target: Entity, world: &mut World) {
        let asset_server = world.get_resource::<AssetServer>().unwrap().clone();
        let mut entity_comamnds = world.entity_mut(target);

        let moon_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("moon.glb"));
        let planet_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("planet.glb"));
        entity_comamnds.insert((Planet, self.transform));

        match self.moon_of {
            Some(m) => {
                entity_comamnds.insert((SceneRoot(moon_handle), m));
            }
            None => {
                entity_comamnds.insert(SceneRoot(planet_handle));
            }
        }
        if let Some(star_of) = self.star_of {
            entity_comamnds.insert(star_of);
        }
    }

    fn extract(builder: SnapshotBuilder) -> SnapshotBuilder {
        builder.extract_prefab(|entity| {
            Some(PlanetPrefab {
                transform: *entity.get::<Transform>()?,
                moon_of: entity.get::<MoonOf>().cloned(),
                star_of: entity.get::<StellarBodyOf>().cloned(),
            })
        })
    }
}

#[derive(Reflect, Debug)]
#[reflect(MapEntities)]
pub struct StarPrefab {
    transform: Transform,
}

impl MapEntities for StarPrefab {
    fn map_entities<E: EntityMapper>(&mut self, entity_mapper: &mut E) {}
}

impl Prefab for StarPrefab {
    type Marker = Star;

    fn spawn(self, target: Entity, world: &mut World) {
        let asset_server = world.get_resource::<AssetServer>().unwrap().clone();
        let mut entity_comamnds = world.entity_mut(target);

        let star_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("star.glb"));
        entity_comamnds.insert((Planet, self.transform));

        entity_comamnds.insert(SceneRoot(star_handle));
    }

    fn extract(builder: SnapshotBuilder) -> SnapshotBuilder {
        builder.extract_prefab(|entity| {
            Some(StarPrefab {
                transform: *entity.get::<Transform>()?,
            })
        })
    }
}
