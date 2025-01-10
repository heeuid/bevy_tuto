use bevy::prelude::*;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash)]
enum AssetName {
    Dice,
    Dices,
    DicesLayout,
}

#[derive(Resource, Default)]
struct AssetTable(HashMap<AssetName, UntypedHandle>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<AssetTable>()
        .add_systems(
            Startup,
            (
                spawn_2d_camera,
                load_asset_dice_png,
                spawn_2d_sprite_dice.after(load_asset_dice_png),
                spawn_2d_sprite_dice_atlas.after(load_asset_dice_png),
            ),
        )
        .run();
}

fn spawn_2d_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn load_asset_dice_png(asset_server: Res<AssetServer>, mut asset_table: ResMut<AssetTable>) {
    let image: Handle<Image> = asset_server.load("dice.png");
    asset_table.0.insert(AssetName::Dice, image.untyped());
    let image: Handle<Image> = asset_server.load("dices.png");
    asset_table.0.insert(AssetName::Dices, image.untyped());
    let layout: Handle<TextureAtlasLayout> = asset_server.add(TextureAtlasLayout::from_grid(
        UVec2::new(110, 123),
        2,
        1,
        None,
        None,
    ));
    asset_table
        .0
        .insert(AssetName::DicesLayout, layout.untyped());
}

fn spawn_2d_sprite_dice(mut commands: Commands, asset_table: Res<AssetTable>) {
    let dice_handle = asset_table.0.get(&AssetName::Dice);
    println!("spawn_2d_sprite_dice");
    if let Some(image_handle) = dice_handle {
        println!("spawning image handle");
        commands.spawn((
            Sprite {
                image: image_handle.clone().typed(),
                // color: Color::srgba(1., 0., 0., 0.1),
                ..default()
            },
            Transform::from_xyz(165. + 89., 0., 0.),
        ));
    }
}

fn spawn_2d_sprite_dice_atlas(mut commands: Commands, asset_table: Res<AssetTable>) {
    let dices_handle = asset_table.0.get(&AssetName::Dices);
    let layout_handle = asset_table.0.get(&AssetName::DicesLayout);
    println!("spawn_2d_sprite_dice_atlas");
    if let (Some(image_handle), Some(layout)) = (dices_handle, layout_handle) {
        println!("spawning sprites");
        let mut atlas = TextureAtlas {
            layout: layout.clone().typed(),
            index: 0,
        };
        commands.spawn((
            Sprite {
                image: image_handle.clone().typed(),
                texture_atlas: Some(atlas.clone()),
                ..default()
            },
            Transform::from_xyz(0., 0., 0.),
        ));

        atlas.index = 1;
        commands.spawn((
            Sprite {
                image: image_handle.clone().typed(),
                texture_atlas: Some(atlas.clone()),
                ..default()
            },
            Transform::from_xyz(110., 0., 0.),
        ));
    }
}
