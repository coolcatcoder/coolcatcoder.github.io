use std::collections::HashMap;

use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    color::palettes::css::{BLACK, LIGHT_BLUE, LIGHT_CORAL, LIGHT_GREEN, PINK, PURPLE, RED, YELLOW},
    prelude::*,
    window::{PrimaryWindow, WindowRef, WindowResolution},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn main(width: u32, height: u32) {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        canvas: Some("#game".into()),
                        resolution: WindowResolution::new(width, height),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            plugin,
        ))
        .run();
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, start)
        .add_systems(
            Update,
            (
                cursor_translation,
                sprite_picker,
                draw_selected,
                picker_sprite_hover_gizmo,
                (draw_cursor,).after(cursor_translation),
            ),
        )
        .init_resource::<CursorGridTranslation>()
        .init_resource::<CursorWorldTranslation>()
        .init_resource::<Tiles>()
        .init_resource::<HoveredPickerSprite>()
        .insert_resource(SpritePickingSettings {
            require_markers: false,
            picking_mode: SpritePickingMode::BoundingBox,
        })
        .add_observer(place);
}

const CELL_LENGTH: f32 = 50.;

fn start(
    mut commands: Commands,
    primary_window: Single<(Entity, &Window), With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((
        Camera2d,
        CameraOnPrimaryWindow,
        RenderTarget::Window(WindowRef::Primary),
        RenderLayers::layer(0),
    ));
    commands
        .entity(primary_window.0)
        .observe(place_press)
        .observe(place_drag)
        .observe(rotation);

    let roster: Vec<(Handle<TextureAtlasLayout>, u16, Vec<Entity>)> = [TextureAtlasLayout::from_grid(
        UVec2::splat(32),
        20,
        32,
        Some(UVec2::splat(1)),
        Some(UVec2::splat(2)),
    )]
    .into_iter()
    .map(|texture_atlas_layout| {
        let rotations = texture_atlas_layout.len();
        (
            texture_atlas_layouts.add(texture_atlas_layout),
            rotations as u16,
            (0..rotations).map(|_| commands.spawn_empty().id()).collect(),
        )
    })
    .collect();

    let texture = asset_server.load("board.png");

    // This is an abandoned idea, as it requires webgpu. On webgl2 you get only one window.
    // let secondary_window = commands.spawn(Window {
    //     resolution: WindowResolution::new(1472, 1550),
    //     canvas: Some("#picker".into()),
    //     ..default()
    // }).id();
    // commands.spawn((Camera2d, RenderTarget::Window(WindowRef::Entity(secondary_window)), RenderLayers::layer(1)));

    commands.spawn((Sprite::from_image(texture.clone()), RenderLayers::layer(1)));

    let mut sprite = Sprite::from_atlas_image(
        texture.clone(),
        TextureAtlas {
            layout: roster[0].0.clone(),
            index: 0,
        },
    );
    sprite.custom_size = Some(Vec2::splat(CELL_LENGTH));
    commands.spawn((CursorSprite, sprite));

    let sprites = Sprites {
        selected: 0,
        rotation: 0,
        roster,
        texture,
    };

    let font = asset_server.load("domine_regular.ttf");
    spawn_picker(&mut commands, font, &sprites, primary_window.1);

    commands.insert_resource(sprites);
}

#[derive(Resource, Default)]
struct CursorGridTranslation(Option<IVec2>);

#[derive(Resource, Default)]
struct CursorWorldTranslation(Option<Vec2>);

fn grid_to_world(grid_translation: IVec2) -> Vec2 {
    grid_translation.as_vec2() * CELL_LENGTH + (CELL_LENGTH * 0.5)
}

#[derive(Component)]
struct CameraOnPrimaryWindow;

fn cursor_translation(
    mut cursor_grid_translation: ResMut<CursorGridTranslation>,
    mut cursor_world_translation: ResMut<CursorWorldTranslation>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<CameraOnPrimaryWindow>>,
) {
    let previous = cursor_grid_translation.0;

    let (camera, camera_transform) = *camera;

    cursor_world_translation.0 = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .map(|ray| ray.origin.xy());

    cursor_grid_translation.bypass_change_detection().0 =
        cursor_world_translation.0.map(|cursor_world_translation| {
            let nearest_cell = (cursor_world_translation / CELL_LENGTH).floor();
            nearest_cell.as_ivec2()
        });

    if previous != cursor_grid_translation.0 {
        cursor_grid_translation.set_changed();
    }
}

#[derive(Component)]
#[require(Transform, Visibility::Hidden)]
struct CursorSprite;

fn draw_cursor(
    mut cursor_sprite: Single<(&mut Transform, &mut Visibility), With<CursorSprite>>,
    cursor_grid_translation: Res<CursorGridTranslation>,
    mut gizmos: Gizmos,
    ui: Query<(&UiHover, &UiDragged)>,
) {
    let Some(cursor_grid_translation) = cursor_grid_translation.0 else {
        *cursor_sprite.1 = Visibility::Hidden;
        return;
    };
    if !ui
        .iter()
        .all(|(hovered, dragged)| !(hovered.0 || dragged.0))
    {
        *cursor_sprite.1 = Visibility::Hidden;
        return;
    }

    *cursor_sprite.1 = Visibility::Visible;

    let cursor_world_translation = grid_to_world(cursor_grid_translation);
    cursor_sprite.0.translation = cursor_world_translation.extend(1.);

    gizmos.rect_2d(cursor_world_translation, Vec2::splat(CELL_LENGTH), RED);
}

#[derive(Resource)]
struct Sprites {
    selected: u16,
    rotation: u16,
    // TO DO: Remove the u16.
    /// (layout, length, tiles on the picker)
    roster: Vec<(Handle<TextureAtlasLayout>, u16, Vec<Entity>)>,
    texture: Handle<Image>,
}

#[derive(Resource, Default)]
struct Tiles(HashMap<IVec2, Entity>);

// Situations we want to place on:
// Mouse down start.
// Mouse down and change of CursorGridTranslation.

#[derive(Event)]
struct Place;

fn place_drag(
    _: On<Pointer<Drag>>,
    cursor_grid_translation: Res<CursorGridTranslation>,
    mut commands: Commands,
) {
    if !cursor_grid_translation.is_changed() {
        return;
    }
    commands.trigger(Place);
}

fn place_press(_: On<Pointer<Press>>, mut commands: Commands) {
    commands.trigger(Place);
}

fn place(
    _: On<Place>,
    mut commands: Commands,
    sprites: Res<Sprites>,
    cursor_grid_translation: Res<CursorGridTranslation>,
    mut tiles: ResMut<Tiles>,
    ui: Query<(&UiHover, &UiDragged)>,
) {
    let Some(cursor_grid_translation) = cursor_grid_translation.0 else {
        return;
    };
    if !ui
        .iter()
        .all(|(hovered, dragged)| !(hovered.0 || dragged.0))
    {
        return;
    }

    info!("Placed.");

    let mut sprite = Sprite::from_atlas_image(
        sprites.texture.clone(),
        TextureAtlas {
            layout: sprites.roster[sprites.selected as usize].0.clone(),
            index: sprites.rotation as usize,
        },
    );

    sprite.custom_size = Some(Vec2::splat(CELL_LENGTH));

    let tile = commands
        .spawn((
            Transform::from_translation(grid_to_world(cursor_grid_translation).extend(0.)),
            sprite,
        ))
        .id();

    if let Some(previous_tile) = tiles.0.insert(cursor_grid_translation, tile) {
        commands.entity(previous_tile).despawn();
        info!("Removed previous.");
    }
}

fn rotation(
    on: On<Pointer<Scroll>>,
    mut sprites: ResMut<Sprites>,
    mut cursor_sprite: Single<&mut Sprite, With<CursorSprite>>,
) {
    let rotation = sprites.rotation as i16 + on.y.signum() as i16;
    let rotation = rotation.rem_euclid(sprites.roster[sprites.selected as usize].1 as i16);
    info!("{}", rotation);
    sprites.rotation = rotation as u16;

    let mut sprite = Sprite::from_atlas_image(
        sprites.texture.clone(),
        TextureAtlas {
            layout: sprites.roster[sprites.selected as usize].0.clone(),
            index: sprites.rotation as usize,
        },
    );
    sprite.custom_size = Some(Vec2::splat(CELL_LENGTH));
    **cursor_sprite = sprite;
}

fn sprite_picker(
    sprite_pickers: Query<(&SpritePicker, &Interaction), Changed<Interaction>>,
    mut sprites: ResMut<Sprites>,
    mut cursor_sprite: Single<&mut Sprite, With<CursorSprite>>,
) {
    let Some(sprite_picker) = sprite_pickers
        .iter()
        .find_map(|(sprite_picker, interaction)| {
            if matches!(interaction, Interaction::Pressed) {
                return Some(sprite_picker);
            }
            None
        })
    else {
        return;
    };

    sprites.selected = sprite_picker.roster_index;
    sprites.rotation = sprite_picker.rotation;

    let mut sprite = Sprite::from_atlas_image(
        sprites.texture.clone(),
        TextureAtlas {
            layout: sprites.roster[sprites.selected as usize].0.clone(),
            index: sprites.rotation as usize,
        },
    );
    sprite.custom_size = Some(Vec2::splat(CELL_LENGTH));
    **cursor_sprite = sprite;

    info!("Selected a new sprite.");
}

#[derive(Component)]
struct UiRoot;

#[derive(Component)]
struct SpritePicker {
    roster_index: u16,
    rotation: u16,
}

fn spawn_ui(commands: &mut Commands, _font: Handle<Font>, sprites: &Sprites) {
    commands
        .spawn((
            UiRoot,
            Interaction::None,
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            Node {
                width: px(800),
                height: percent(50),
                display: Display::Grid,
                padding: px(3).all(),
                grid_template_columns: RepeatedGridTrack::flex(25, 1.),
                //grid_template_rows: RepeatedGridTrack::flex(21, 1.),
                grid_auto_flow: GridAutoFlow::Row,
                row_gap: px(3),
                column_gap: px(3),
                align_self: AlignSelf::End,
                ..default()
            },
        ))
        .with_children(|builder| {
            sprites
                .roster
                .iter()
                .enumerate()
                .flat_map(|(roster_index, (texture_atlas_layout, length, _))| {
                    (0..*length).map(move |index| {
                        (
                            SpritePicker {
                                roster_index: roster_index as u16,
                                rotation: index,
                            },
                            Node {
                                //margin: Val::Percent(1.).all(),
                                //height: px(5),
                                aspect_ratio: Some(1.),
                                ..default()
                            },
                            Interaction::None,
                            ImageNode::from_atlas_image(
                                sprites.texture.clone(),
                                TextureAtlas {
                                    layout: texture_atlas_layout.clone(),
                                    index: index as usize,
                                },
                            ),
                        )
                    })
                })
                .for_each(|bundle| {
                    builder.spawn(bundle);
                });
        });
}

fn tile_picker_ui(commands: &mut Commands, font: Handle<Font>, sprites: &Sprites) -> impl Bundle {
    const FONT_SIZE: f32 = 20.;
    const LINE_HEIGHT: f32 = 45.;

    let list = sprites.roster.iter().enumerate().flat_map(
        |(roster_index, (texture_atlas_layout, length, _))| {
            (0..*length).map(move |index| {
                (
                    Node {
                        min_height: px(LINE_HEIGHT),
                        max_height: px(LINE_HEIGHT),
                        ..default()
                    },
                    children![(
                        // Text(format!("Item {i}")),
                        // TextFont {
                        //     font: font.clone(),
                        //     ..default()
                        // },
                        // Label,
                        SpritePicker {
                            roster_index: roster_index as u16,
                            rotation: index,
                        },
                        Node {
                            margin: Val::Percent(1.).all(),
                            ..default()
                        },
                        Interaction::None,
                        ImageNode::from_atlas_image(
                            sprites.texture.clone(),
                            TextureAtlas {
                                layout: texture_atlas_layout.clone(),
                                index: index as usize,
                            }
                        ),
                        //AccessibilityNode(accesskit::Node::new(Role::ListItem)),
                    )],
                )
            })
        },
    );

    let parent = commands
        .spawn((
            UiRoot,
            Interaction::None,
            Node {
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                width: px(200),
                ..default()
            },
        ))
        .id();

    let child = commands
        .spawn((
            // Title
            Text::new("Picker!"),
            TextFont {
                font: font.clone(),
                font_size: FONT_SIZE,
                ..default()
            },
            Label,
        ))
        .id();
    commands.entity(parent).add_child(child);

    let child = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::Stretch,
                height: percent(50),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.10, 0.10, 0.10)),
        ))
        .id();
    commands.entity(parent).add_child(child);

    let mut parent = commands.entity(child);
    for bundle in list {
        parent.with_child(bundle);
    }
}

#[derive(Event)]
struct PickerUpdate {
    /// The centre of the grab.
    translation: Vec2,
    resize_delta: Vec2,
}

#[derive(Component)]
struct UiHover(bool);

#[derive(Component)]
struct UiDragged(bool);

const PICKER_TILE_LENGTH: f32 = CELL_LENGTH * 0.5;

fn spawn_picker(commands: &mut Commands, _font: Handle<Font>, sprites: &Sprites, window: &Window) {
    let tiles_to_pick: Vec<Entity> = sprites
        .roster
        .iter()
        .enumerate()
        .flat_map(|(roster_index, (texture_atlas_layout, _, entities))| {
            entities.iter().copied().enumerate().map(move |(index, entity)| {
                let mut sprite = Sprite::from_atlas_image(
                    sprites.texture.clone(),
                    TextureAtlas {
                        layout: texture_atlas_layout.clone(),
                        index,
                    },
                );
                sprite.custom_size = Some(Vec2::splat(PICKER_TILE_LENGTH));
                (entity, (
                    sprite,
                    SpritePicker {
                        roster_index: roster_index as u16,
                        rotation: index as u16,
                    },
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: true,
                    },
                ))
            })
        })
        .map(|(entity, bundle)| commands.entity(entity).insert(bundle).observe(picker_sprite_over).observe(picker_sprite_out).observe(|on: On<Pointer<Click>>, mut sprite_picker: Query<&SpritePicker>, mut sprites: ResMut<Sprites>, mut cursor_sprite: Single<&mut Sprite, With<CursorSprite>>| {
            let sprite_picker = sprite_picker.get_mut(on.entity).unwrap();

            sprites.selected = sprite_picker.roster_index;
            sprites.rotation = sprite_picker.rotation;

            let mut sprite = Sprite::from_atlas_image(
                sprites.texture.clone(),
                TextureAtlas {
                    layout: sprites.roster[sprites.selected as usize].0.clone(),
                    index: sprites.rotation as usize,
                },
            );
            sprite.custom_size = Some(Vec2::splat(CELL_LENGTH));
            **cursor_sprite = sprite;

            info!("Selected a new sprite.");
        }).id())
        .collect();

    let [base, grab, resize] = [BLACK, LIGHT_BLUE, LIGHT_GREEN].map(|colour| {
        commands
            .spawn((
                UiHover(false),
                UiDragged(false),
                Sprite::from_color(colour, Vec2::ZERO),
                Pickable {
                    should_block_lower: true,
                    is_hoverable: true,
                },
            ))
            .observe(|on: On<Pointer<Over>>, mut hover: Query<&mut UiHover>| {
                hover.get_mut(on.entity).unwrap().0 = true;
            })
            .observe(|on: On<Pointer<Out>>, mut hover: Query<&mut UiHover>| {
                hover.get_mut(on.entity).unwrap().0 = false;
            })
            .id()
    });

    commands.add_observer(
        move |on: On<PickerUpdate>,
              mut sprites: Query<(&mut Transform, &mut Sprite)>,
              mut visibility: Query<&mut Visibility>| {
            let (mut base_transform, mut base_sprite) = sprites.get_mut(base).unwrap();
            let size = base_sprite.custom_size.unwrap() + on.resize_delta;
            base_transform.translation = Vec3::new(
                on.translation.x,
                on.translation.y - size.y * 0.5 - CELL_LENGTH * 0.5,
                1.,
            );
            base_sprite.custom_size = Some(size);

            // grab
            let (mut transform, mut sprite) = sprites.get_mut(grab).unwrap();
            transform.translation = on.translation.extend(2.);
            sprite.custom_size = Some(Vec2::new(size.x, CELL_LENGTH));

            // resize
            let (mut transform, mut sprite) = sprites.get_mut(resize).unwrap();
            transform.translation = Vec3::new(
                on.translation.x,
                on.translation.y - size.y - CELL_LENGTH,
                2.,
            );
            sprite.custom_size = Some(Vec2::new(size.x, CELL_LENGTH));

            // tiles
            let margin = 10.;
            let space_needed_for_single_tile = margin + PICKER_TILE_LENGTH;

            let top_left_corner = on.translation + Vec2::new(size.x * -0.5, CELL_LENGTH * -0.5);
            let mut width_remaining = size.x;
            let mut height_remaining = size.y - space_needed_for_single_tile;

            let mut tiles_to_pick_iter = tiles_to_pick.iter();

            for entity in tiles_to_pick_iter.by_ref() {
                if width_remaining < space_needed_for_single_tile {
                    if height_remaining < space_needed_for_single_tile {
                        *visibility.get_mut(*entity).unwrap() = Visibility::Hidden;
                        break;
                    } else {
                        width_remaining = size.x;
                        height_remaining -= space_needed_for_single_tile;
                    }
                }

                width_remaining -= space_needed_for_single_tile;

                sprites.get_mut(*entity).unwrap().0.translation = Vec3::new(
                    top_left_corner.x + (size.x - width_remaining) - PICKER_TILE_LENGTH * 0.5,
                    top_left_corner.y - (size.y - height_remaining) + PICKER_TILE_LENGTH * 0.5,
                    3.,
                );
                *visibility.get_mut(*entity).unwrap() = Visibility::Visible;
            }

            for entity in tiles_to_pick_iter {
                *visibility.get_mut(*entity).unwrap() = Visibility::Hidden;
            }
        },
    );

    commands
        .entity(grab)
        .observe(
            |_: On<Pointer<Drag>>,
             cursor_world_translation: Res<CursorWorldTranslation>,
             mut commands: Commands| {
                let Some(cursor_world_translation) = cursor_world_translation.0 else {
                    return;
                };

                commands.trigger(PickerUpdate {
                    translation: cursor_world_translation,
                    resize_delta: Vec2::ZERO,
                });
            },
        )
        .observe(
            |on: On<Pointer<DragStart>>, mut drag: Query<&mut UiDragged>| {
                drag.get_mut(on.entity).unwrap().0 = true;
            },
        )
        .observe(
            |on: On<Pointer<DragEnd>>, mut drag: Query<&mut UiDragged>| {
                drag.get_mut(on.entity).unwrap().0 = false;
            },
        );

    commands
        .entity(resize)
        .observe(
            move |on: On<Pointer<Drag>>,
                  translations: Query<&Transform>,
                  mut commands: Commands| {
                commands.trigger(PickerUpdate {
                    translation: translations.get(grab).unwrap().translation.xy()
                        + Vec2::X * on.delta.x * 0.5,
                    resize_delta: on.delta,
                });
            },
        )
        .observe(
            |on: On<Pointer<DragStart>>, mut drag: Query<&mut UiDragged>| {
                drag.get_mut(on.entity).unwrap().0 = true;
            },
        )
        .observe(
            |on: On<Pointer<DragEnd>>, mut drag: Query<&mut UiDragged>| {
                drag.get_mut(on.entity).unwrap().0 = false;
            },
        );

    let size = Vec2::splat(CELL_LENGTH) * 14. + Vec2::splat(10.);
    commands.trigger(PickerUpdate {
        translation: Vec2::new(window.width() * -0.5 + size.x * 0.5, window.height() * 0.5 - CELL_LENGTH * 0.5),
        resize_delta: size,
    });
}

#[derive(Resource, Default)]
struct HoveredPickerSprite(Option<Entity>);

fn picker_sprite_over(on: On<Pointer<Over>>, mut hovered_picker_sprite: ResMut<HoveredPickerSprite>) {
    hovered_picker_sprite.0 = Some(on.entity);
}

fn picker_sprite_out(on: On<Pointer<Out>>, mut hovered_picker_sprite: ResMut<HoveredPickerSprite>) {
    if let Some(entity) = hovered_picker_sprite.0 && entity == on.entity {
        hovered_picker_sprite.0 = None;
    }
}

fn picker_sprite_hover_gizmo(hovered_picker_sprite: Res<HoveredPickerSprite>, transforms: Query<&Transform>, mut gizmos: Gizmos) {
    let Some(entity) = hovered_picker_sprite.0 else {
        return;
    };

    let translation = transforms.get(entity).unwrap().translation.xy();
    gizmos.rect_2d(translation, Vec2::splat(PICKER_TILE_LENGTH), RED);
}

fn draw_selected(sprites: Res<Sprites>, transforms: Query<(&Transform, &Visibility)>, mut gizmos: Gizmos) {
    let (transform, visibility) = transforms.get(sprites.roster[sprites.selected as usize].2[sprites.rotation as usize]).unwrap();
    if matches!(visibility, Visibility::Hidden) {
        return;
    }

    let translation = transform.translation.xy();

    gizmos.rect_2d(translation, Vec2::splat(PICKER_TILE_LENGTH), RED);
    gizmos.rect_2d(translation, Vec2::splat(PICKER_TILE_LENGTH + 5.), YELLOW);
    gizmos.rect_2d(translation, Vec2::splat(PICKER_TILE_LENGTH + 10.), PURPLE);
}
