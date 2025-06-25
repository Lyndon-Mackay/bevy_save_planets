mod save;

use std::ascii::AsciiExt;
use std::time::Duration;

use bevy::time::{self, common_conditions};
use bevy::{
    core_pipeline::bloom::Bloom, input::common_conditions::input_toggle_active, prelude::*,
};

use bevy::ecs::entity::MapEntities;
use bevy::ecs::reflect::ReflectMapEntities;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_save::ext::WorldSaveableExt;
use save::{PlanetPipeline, PlanetSavePlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F7)))
        .add_plugins(PlanetSavePlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (check_moon_status, check_star_status)
                .run_if(common_conditions::on_timer(Duration::from_millis(500))),
        )
        .register_type::<Planet>()
        .register_type::<MoonOf>()
        .register_type::<OrbitingMoons>()
        .register_type::<Star>()
        .register_type::<StellarBodies>()
        .register_type::<StellarBodyOf>()
        .run();
}

#[derive(Component, Clone, Copy, Default, Reflect)]
pub struct Planet;

#[derive(Component, Clone, Debug, MapEntities, Reflect)]
#[relationship(relationship_target = OrbitingMoons)]
#[reflect(Component, Clone, MapEntities)]
pub struct MoonOf(#[entities] pub Entity);

#[derive(Component, Clone, Default, MapEntities, Reflect)]
#[relationship_target(relationship = MoonOf)]
#[reflect(Component, Clone, MapEntities)]
pub struct OrbitingMoons(#[entities] Vec<Entity>);

#[derive(Component, Clone, Default)]
struct MoonStatus;

#[derive(Component, Clone, Copy, Default, Reflect)]
pub struct Star;

#[derive(Component, Clone, Debug, MapEntities, Reflect)]
#[relationship(relationship_target = StellarBodies)]
#[reflect(Component, Clone, MapEntities)]
pub struct StellarBodyOf(#[entities] pub Entity);

#[derive(Component, Clone, Default, MapEntities, Reflect)]
#[relationship_target(relationship = StellarBodyOf)]
#[reflect(Component, Clone, MapEntities)]
pub struct StellarBodies(#[entities] Vec<Entity>);

#[derive(Component, Clone, Default)]
struct StarStatus;
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera_transform = Transform::from_xyz(0.0, 73.0, 13.0).looking_at(Vec3::ZERO, Vec3::Y);

    let bloom = Bloom {
        intensity: 0.2,
        ..default()
    };

    commands.insert_resource(AmbientLight {
        brightness: 15.0,
        ..Default::default()
    });

    commands.insert_resource(ClearColor(Color::BLACK));

    commands.spawn((
        Camera3d::default(),
        Camera {
            hdr: true,
            ..Default::default()
        },
        camera_transform,
        bloom,
    ));

    //moon status
    commands
        .spawn((
            Node::default(),
            BackgroundColor(Color::Srgba(Srgba::rgb(0.0, 0.0, 1.0))),
        ))
        .with_children(|c| {
            c.spawn((MoonStatus, Text("I have a moon :)".to_string())));
        });

    //star status
    commands
        .spawn((
            Node {
                top: Val::Percent(30.0),
                ..Default::default()
            },
            BackgroundColor(Color::Srgba(Srgba::rgb(0.0, 0.0, 1.0))),
        ))
        .with_children(|c| {
            c.spawn((StarStatus, Text("I have a star XD".to_string())));
        });
    //save button
    commands
        .spawn((
            Node {
                left: Val::Percent(30.0),
                ..Default::default()
            },
            BackgroundColor(Color::Srgba(Srgba::rgb(0.0, 0.0, 1.0))),
        ))
        .with_children(|c| {
            c.spawn(Text("Save".to_string()));
        })
        .observe(save_pressed);

    //load_button
    commands
        .spawn((
            Node {
                left: Val::Percent(70.0),
                ..Default::default()
            },
            BackgroundColor(Color::Srgba(Srgba::rgb(0.0, 0.0, 1.0))),
        ))
        .with_children(|c| {
            c.spawn(Text("Load".to_string()));
        })
        .observe(load_pressed);

    let star_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("star.glb"));

    let star_transform = Transform::from_xyz(12.0, 0.0, 13.0).with_scale(Vec3::splat(0.2));
    let star_entity = commands
        .spawn((Star, SceneRoot(star_handle), star_transform))
        .id();

    let planet_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("planet.glb"));
    let planet_transform = Transform::from_scale(Vec3::splat(0.15));
    let planet_entity = commands
        .spawn((
            Planet,
            SceneRoot(planet_handle),
            planet_transform,
            StellarBodyOf(star_entity),
        ))
        .id();

    let moon_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("moon.glb"));
    let moon_transform = Transform::from_xyz(-8.0, 23.0, 13.0).with_scale(Vec3::splat(0.05));
    commands.spawn((
        Planet,
        SceneRoot(moon_handle),
        moon_transform,
        MoonOf(planet_entity),
        StellarBodyOf(star_entity),
    ));
}

fn save_pressed(trigger: Trigger<Pointer<Pressed>>, mut commands: Commands) {
    commands.run_system_cached(save);
}

fn load_pressed(trigger: Trigger<Pointer<Pressed>>, mut commands: Commands) {
    commands.run_system_cached(load);
}

fn save(world: &mut World) {
    world.save(PlanetPipeline).unwrap();
}

fn load(world: &mut World) {
    world.load(PlanetPipeline).unwrap();
}

fn check_moon_status(mut text: Query<&mut Text, With<MoonStatus>>, moon_query: Query<&MoonOf>) {
    let mut text = text.single_mut().unwrap();
    if moon_query.single().is_ok() {
        text.0 = "I have a moon XD".to_string();
    } else {
        text.0 = "I want my moon back >:[".to_string();
    }
}
fn check_star_status(
    mut text: Query<&mut Text, With<StarStatus>>,
    star_query: Query<&StellarBodyOf>,
) {
    let mut text = text.single_mut().unwrap();
    if star_query.iter().count() > 1 {
        text.0 = "I have a star XD".to_string();
    } else {
        text.0 = "I want my star back >:[".to_string();
    }
}
