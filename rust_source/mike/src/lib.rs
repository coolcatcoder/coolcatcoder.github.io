#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use std::{collections::HashMap, ops::Range};

use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    color::palettes::css::{
        BLACK, BROWN, GREEN, LIGHT_BLUE, LIGHT_GREEN, PURPLE, RED, WHITE, YELLOW,
    },
    diagnostic::FrameCount,
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
                draw_selected,
                picker_sprite_hover_gizmo,
                display_dialogue,
                game_state,
                (draw_cursor,).after(cursor_translation),
            ),
        )
        .init_resource::<InteractionTranslation>()
        .init_resource::<CursorTranslation>()
        .init_resource::<Tiles>()
        .init_resource::<HoveredPickerSprite>()
        .init_resource::<GameState>()
        .init_resource::<Mode>()
        .insert_resource(SpritePickingSettings {
            require_markers: false,
            picking_mode: SpritePickingMode::BoundingBox,
        })
        .add_observer(on_press)
        .add_observer(on_drag)
        .add_observer(place)
        .add_observer(erase)
        .add_observer(next_dialogue)
        .init_resource::<Dialogue>();
}

const CELL_LENGTH: f32 = 50.;

fn start(
    mut commands: Commands,
    primary_window: Single<(Entity, &Window), With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut dialogue: ResMut<Dialogue>,
) {
    commands.spawn((
        Camera2d,
        CameraOnPrimaryWindow,
        RenderTarget::Window(WindowRef::Primary),
        RenderLayers::layer(0),
    ));
    commands
        .entity(primary_window.0)
        .observe(window_on_press)
        .observe(window_on_drag)
        .observe(rotation);

    let texture_atlas_layouts: Vec<Handle<TextureAtlasLayout>> = [
        TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            20,
            32,
            Some(UVec2::splat(1)),
            Some(UVec2::splat(2)),
        ),
        TextureAtlasLayout::from_grid(
            UVec2::splat(16),
            2,
            1,
            Some(UVec2::splat(1)),
            Some(UVec2::new(2 + 32 * 20 + 22, 2)),
        ),
        TextureAtlasLayout::from_grid(
            UVec2::splat(16),
            3,
            1,
            Some(UVec2::splat(3)),
            Some(UVec2::new(1002, 154)),
        ),
        TextureAtlasLayout::from_grid(UVec2::splat(16), 3, 1, None, Some(UVec2::new(1059, 154))),
        TextureAtlasLayout::from_grid(UVec2::splat(16), 5, 1, None, Some(UVec2::new(1110, 154))),
        TextureAtlasLayout::from_grid(UVec2::splat(16), 5, 1, None, Some(UVec2::new(1193, 154))),
    ]
    .into_iter()
    .map(|layout| texture_atlas_layouts.add(layout))
    .collect();

    #[allow(clippy::single_range_in_vec_init)]
    let tabs: [&[(usize, bool, &[Range<usize>])]; _] = [
        &[(0, false, &[0..280])],
        &[
            (0, false, &[280..539]),
            (2, false, &[0..3]),
            (3, false, &[0..3]),
            (4, false, &[0..4]),
            (5, false, &[0..5]),
            (5, true, &[0..5]),
            (2, true, &[2..3]),
        ],
    ];
    let tabs: Vec<Vec<(usize, usize, Entity, bool)>> = tabs
        .into_iter()
        .map(|sprites| {
            sprites
                .iter()
                .flat_map(|(layout, flip_x, sprite_ranges)| {
                    sprite_ranges.iter().cloned().flat_map(move |sprite_range| {
                        sprite_range.map(move |sprite_index| (layout, sprite_index, *flip_x))
                    })
                })
                .map(|(layout, sprite, flip_x)| {
                    (*layout, sprite, commands.spawn_empty().id(), flip_x)
                })
                .collect()
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

    //commands.spawn((Sprite::from_image(texture.clone()), RenderLayers::layer(1)));

    let mut sprite = Sprite::from_atlas_image(
        texture.clone(),
        TextureAtlas {
            layout: texture_atlas_layouts[tabs[0][0].0].clone(),
            index: 0,
        },
    );
    sprite.custom_size = Some(Vec2::splat(CELL_LENGTH));
    commands.spawn((CursorSprite, sprite));

    let sprites = Sprites {
        tab: 0,
        selected: 0,
        texture_atlas_layouts,
        tabs,
        texture,
    };

    let font = asset_server.load("domine_regular.ttf");
    spawn_picker(&mut commands, font, &sprites, primary_window.1);

    commands.insert_resource(sprites);

    spawn_dialogue_box(&mut commands, &asset_server);

    dialogue.set([
        "Hey Mike!\n(You click the dialogue box to continue.)",
        "You see that grid of tiles on the left?\nClick a tile to select it.\nYou can also scroll to navigate.",
        "The blue bar at the top moves the grid.\nThe green bar at the bottom resizes\nthe grid.",
        "Tabs are to the right of the grid.\nClick a tab to open its category of tiles.\nThe longest tab is the one currently open.",
        "Once you have selected a tile, you can\nthen draw in the empty space to the right.\nLeft click to place.\nRight click to erase.",
        "Your job is to assemble the various\nboards, as accurately as possible.",
        "You will be shown the blueprint for the\nboard for an amount of time, before it\ndisappears.",
        "When you are finished press the \"DONE\"\nbutton in the bottom right.\n(It will appear when the game starts.)",
        "Good luck! Have fun!",
    ], |state| *state.0 = GameState::REST);
}

#[derive(Resource, Default)]
struct InteractionTranslation {
    screen: Vec2,
    /// Will be identical to screen, unless in the middle of a drag.
    screen_before_drag: Vec2,

    world: Vec2,
    grid: IVec2,
}

#[derive(EntityEvent)]
struct WorldPress {
    entity: Entity,
    translation: Vec2,
    button: PointerButton,
}

#[derive(EntityEvent)]
struct WorldDrag {
    entity: Entity,
    translation: Vec2,
    button: PointerButton,
}

fn on_press(
    mut on: On<Pointer<Press>>,
    mut interaction_translation: ResMut<InteractionTranslation>,
    camera: Single<(&Camera, &GlobalTransform), With<CameraOnPrimaryWindow>>,
    sprite: Query<(), With<Sprite>>,
    mut commands: Commands,
) {
    info!("Pressed. {:?}", on.hit.position);

    if sprite.get(on.entity).is_ok() {
        on.propagate(false);
        info!("Sprite");
        interaction_translation.world = on.hit.position.unwrap().xy();

        let Some(screen_translation) = world_to_screen(interaction_translation.world, *camera)
        else {
            error!("I truly do not know.");
            return;
        };
        interaction_translation.screen = screen_translation;
        interaction_translation.screen_before_drag = screen_translation;
    } else {
        let Some(screen_translation) = on.hit.position.map(|translation| translation.xy()) else {
            return;
        };
        interaction_translation.screen = screen_translation;
        interaction_translation.screen_before_drag = screen_translation;

        let Some(world_translation) = screen_to_world(interaction_translation.screen, *camera)
        else {
            return;
        };
        interaction_translation.world = world_translation;
    }

    commands.trigger(WorldPress {
        entity: on.entity,
        translation: interaction_translation.world,
        button: on.button,
    });
}

fn on_drag(
    on: On<Pointer<Drag>>,
    mut interaction_translation: ResMut<InteractionTranslation>,
    camera: Single<(&Camera, &GlobalTransform), With<CameraOnPrimaryWindow>>,
    mut previous_delta: Local<Vec2>,
    frame_count: Res<FrameCount>,
    mut commands: Commands,
) {
    if on.delta != *previous_delta || frame_count.is_changed() {
        //info!("Changed.");
        *previous_delta = on.delta;
        interaction_translation.screen = interaction_translation.screen_before_drag + on.distance;

        let Some(world_translation) = screen_to_world(interaction_translation.screen, *camera)
        else {
            error!("screen_to_world failed");
            return;
        };
        interaction_translation.world = world_translation;
    }

    //info!("WorldDrag");

    commands.trigger(WorldDrag {
        entity: on.entity,
        translation: interaction_translation.world,
        button: on.button,
    });
}

// Place and Erase trigger when the user interacts with the grid, using their respective buttons.
#[derive(Event)]
struct Place(IVec2);
#[derive(Event)]
struct Erase(IVec2);

impl Place {
    fn world(&self) -> Vec2 {
        grid_to_world(self.0)
    }
}

fn window_on_press(
    on: On<WorldPress>,
    mut interaction_translation: ResMut<InteractionTranslation>,
    cursor_translation: Res<CursorTranslation>,
    mut commands: Commands,
) {
    interaction_translation.grid = world_to_nearest_grid(on.translation);
    match on.button {
        PointerButton::Primary if cursor_translation.grid.is_none() => commands.trigger(Erase(interaction_translation.grid)),
        PointerButton::Primary => commands.trigger(Place(interaction_translation.grid)),
        PointerButton::Secondary => commands.trigger(Erase(interaction_translation.grid)),
        PointerButton::Middle => (),
    }
}

fn window_on_drag(
    on: On<WorldDrag>,
    mut interaction_translation: ResMut<InteractionTranslation>,
    cursor_translation: Res<CursorTranslation>,
    mode: Res<Mode>,
    mut commands: Commands,
) {
    let grid_translation = world_to_nearest_grid(on.translation);

    if grid_translation == interaction_translation.grid {
        return;
    }
    interaction_translation.grid = grid_translation;

    match on.button {
        PointerButton::Primary if cursor_translation.grid.is_none() => match *mode {
            Mode::Place => commands.trigger(Place(interaction_translation.grid)),
            Mode::Erase => commands.trigger(Erase(interaction_translation.grid)),
        }
        PointerButton::Primary => commands.trigger(Place(interaction_translation.grid)),
        PointerButton::Secondary => commands.trigger(Erase(interaction_translation.grid)),
        PointerButton::Middle => (),
    }
}

fn grid_to_world(grid_translation: IVec2) -> Vec2 {
    grid_translation.as_vec2() * CELL_LENGTH + (CELL_LENGTH * 0.5)
}

fn screen_to_world(screen_translation: Vec2, camera: (&Camera, &GlobalTransform)) -> Option<Vec2> {
    let (camera, camera_transform) = camera;
    camera
        .viewport_to_world(camera_transform, screen_translation)
        .ok()
        .map(|ray| ray.origin.xy())
}

fn world_to_screen(world_translation: Vec2, camera: (&Camera, &GlobalTransform)) -> Option<Vec2> {
    let (camera, camera_transform) = camera;
    camera
        .world_to_viewport(camera_transform, world_translation.extend(0.))
        .ok()
}

fn world_to_nearest_grid(world_translation: Vec2) -> IVec2 {
    let nearest_cell = (world_translation / CELL_LENGTH).floor();
    nearest_cell.as_ivec2()
}

#[derive(Component)]
struct CameraOnPrimaryWindow;

#[derive(Resource, Default)]
struct CursorTranslation {
    grid: Option<IVec2>,
}

fn cursor_translation(
    mut cursor_translation: ResMut<CursorTranslation>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<CameraOnPrimaryWindow>>,
) {
    let (camera, camera_transform) = *camera;

    let cursor_world_translation = window
        .cursor_position()
        .and_then(|cursor| screen_to_world(cursor, (camera, camera_transform)));

    cursor_translation.grid = cursor_world_translation.map(world_to_nearest_grid);
}

#[derive(Component)]
#[require(Transform, Visibility::Hidden)]
struct CursorSprite;

fn draw_cursor(
    mut cursor_sprite: Single<(&mut Transform, &mut Visibility), With<CursorSprite>>,
    mut mode_button: Single<&mut Visibility, (With<ModeButton>, Without<CursorSprite>)>,
    cursor_translation: Res<CursorTranslation>,
    mut gizmos: Gizmos,
    ui: Query<(&UiHover, &UiDragged)>,
    interaction_translation: Res<InteractionTranslation>,
) {
    let Some(cursor_grid_translation) = cursor_translation.grid else {
        *cursor_sprite.1 = Visibility::Hidden;
        **mode_button = Visibility::Visible;
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
    **mode_button = Visibility::Hidden;

    let cursor_world_translation = grid_to_world(cursor_grid_translation);
    cursor_sprite.0.translation = cursor_world_translation.extend(1.);

    gizmos.rect_2d(cursor_world_translation, Vec2::splat(CELL_LENGTH), RED);
    #[cfg(target_os = "linux")]
    gizmos.circle_2d(interaction_translation.world, 1., RED);
}

#[derive(Component)]
struct SpriteMarker(usize, usize);

#[derive(Resource)]
struct Sprites {
    tab: usize,
    selected: usize,

    texture_atlas_layouts: Vec<Handle<TextureAtlasLayout>>,
    // (texture_atlas_layouts index, atlas index, picker sprite, flipped x)
    tabs: Vec<Vec<(usize, usize, Entity, bool)>>,
    texture: Handle<Image>,
}

impl Sprites {
    fn sprite(&self) -> Sprite {
        let selected = self.tabs[self.tab][self.selected];

        let mut sprite = Sprite::from_atlas_image(
            self.texture.clone(),
            TextureAtlas {
                layout: self.texture_atlas_layouts[selected.0].clone(),
                index: selected.1,
            },
        );
        sprite.custom_size = Some(Vec2::splat(CELL_LENGTH));
        sprite.flip_x = selected.3;

        sprite
    }

    fn sprite_marker(&self) -> SpriteMarker {
        SpriteMarker(self.tab, self.selected)
    }
}

#[derive(Resource, Default)]
struct Tiles(HashMap<IVec2, Entity>);

fn erase(
    on: On<Erase>,
    mut tiles: ResMut<Tiles>,
    ui: Query<(&UiHover, &UiDragged)>,
    sprite_markers: Query<&SpriteMarker>,
    mut commands: Commands,
) {
    if !ui
        .iter()
        .all(|(hovered, dragged)| !(hovered.0 || dragged.0))
    {
        return;
    }

    info!("Erased.");

    #[cfg(target_os = "linux")]
    {
        let debug_tiles: Vec<(i32, i32, usize, usize)> = tiles
            .0
            .iter()
            .map(|(translation, entity)| {
                let sprite_marker = sprite_markers.get(*entity).unwrap();
                (
                    translation.x,
                    translation.y,
                    sprite_marker.0,
                    sprite_marker.1,
                )
            })
            .collect();
        info!("{:?}", debug_tiles);
    }

    if let Some(previous_tile) = tiles.0.remove(&on.0) {
        commands.entity(previous_tile).despawn();
        info!("Removed previous.");
    }
}

fn place(
    on: On<Place>,
    mut commands: Commands,
    sprites: Res<Sprites>,
    mut tiles: ResMut<Tiles>,
    ui: Query<(&UiHover, &UiDragged)>,
) {
    if !ui
        .iter()
        .all(|(hovered, dragged)| !(hovered.0 || dragged.0))
    {
        return;
    }

    info!("Placed.");

    let tile = commands
        .spawn((
            Transform::from_translation(on.world().extend(0.)),
            sprites.sprite(),
            sprites.sprite_marker(),
        ))
        .id();

    if let Some(previous_tile) = tiles.0.insert(on.0, tile) {
        commands.entity(previous_tile).despawn();
        info!("Removed previous.");
    }
}

fn rotation(
    on: On<Pointer<Scroll>>,
    mut sprites: ResMut<Sprites>,
    mut cursor_sprite: Single<&mut Sprite, With<CursorSprite>>,
) {
    let rotation = sprites.selected as isize + on.y.signum() as isize;
    let rotation = rotation.rem_euclid(sprites.tabs[sprites.tab].len() as isize);
    info!("{}", rotation);
    sprites.selected = rotation as usize;

    **cursor_sprite = sprites.sprite();
}

#[derive(Component)]
struct SpritePicker(usize);

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
    let not_sure: Vec<Vec<Entity>> = sprites
        .tabs
        .iter()
        .map(|tab_sprites| {
            tab_sprites
.iter().copied()
        .enumerate()
        .map(|(index, (layout_index, sprite_index, entity, flip_x))| {

                let mut sprite = Sprite::from_atlas_image(
                    sprites.texture.clone(),
                    TextureAtlas {
                        layout: sprites.texture_atlas_layouts[layout_index].clone(),
                        index: sprite_index,
                    },
                );
                sprite.custom_size = Some(Vec2::splat(PICKER_TILE_LENGTH));
                sprite.flip_x = flip_x;
                (entity, (
                    sprite,
                    SpritePicker(index),
                    Pickable {
                        should_block_lower: false,
                        is_hoverable: true,
                    },
                ))

        })
        .map(|(entity, bundle)| commands.entity(entity).insert(bundle).observe(picker_sprite_over).observe(picker_sprite_out).observe(|on: On<Pointer<Click>>, mut sprite_picker: Query<&SpritePicker>, mut sprites: ResMut<Sprites>, mut cursor_sprite: Single<&mut Sprite, With<CursorSprite>>| {
            let sprite_picker = sprite_picker.get_mut(on.entity).unwrap();

            sprites.selected = sprite_picker.0;

            **cursor_sprite = sprites.sprite();

            info!("Selected a new sprite.");
        }).id())
        .collect()
        })
        .collect();

    fn spawn_ui<'a>(commands: &'a mut Commands, bundle: impl Bundle) -> EntityCommands<'a> {
        let mut entity_commands = commands.spawn((
            bundle,
            UiHover(false),
            UiDragged(false),
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
        ));
        entity_commands
            .observe(|on: On<Pointer<Over>>, mut hover: Query<&mut UiHover>| {
                hover.get_mut(on.entity).unwrap().0 = true;
            })
            .observe(|on: On<Pointer<Out>>, mut hover: Query<&mut UiHover>| {
                hover.get_mut(on.entity).unwrap().0 = false;
            });
        entity_commands
    }

    let [base, grab, resize] = [BLACK, LIGHT_BLUE, LIGHT_GREEN]
        .map(|colour| spawn_ui(commands, Sprite::from_color(colour, Vec2::ZERO)).id());

    let mut tab_index = 0;
    let tabs = [BROWN, GREEN].map(|colour| {
        let entity = spawn_ui(commands, Sprite::from_color(colour, Vec2::ZERO))
            .observe(
                move |_: On<Pointer<Click>>,
                      mut commands: Commands,
                      transforms: Query<&Transform>,
                      mut sprites: ResMut<Sprites>,
                      mut cursor_sprite: Single<&mut Sprite, With<CursorSprite>>| {
                    sprites.tab = tab_index;
                    sprites.selected = 0;

                    commands.trigger(PickerUpdate {
                        translation: transforms.get(grab).unwrap().translation.xy(),
                        resize_delta: Vec2::ZERO,
                    });

                    **cursor_sprite = sprites.sprite();
                },
            )
            .id();
        tab_index += 1;
        entity
    });

    commands.add_observer(
        move |on: On<PickerUpdate>,
              mut transforms: Query<(&mut Transform, &mut Sprite)>,
              mut visibility: Query<&mut Visibility>,
              sprites: Option<Res<Sprites>>| {
            let (mut base_transform, mut base_sprite) = transforms.get_mut(base).unwrap();
            let size = base_sprite.custom_size.unwrap() + on.resize_delta;
            base_transform.translation = Vec3::new(
                on.translation.x,
                on.translation.y - size.y * 0.5 - CELL_LENGTH * 0.5,
                1.,
            );
            base_sprite.custom_size = Some(size);

            // grab
            let (mut transform, mut sprite) = transforms.get_mut(grab).unwrap();
            transform.translation = on.translation.extend(2.);
            sprite.custom_size = Some(Vec2::new(size.x, CELL_LENGTH));

            // resize
            let (mut transform, mut sprite) = transforms.get_mut(resize).unwrap();
            transform.translation = Vec3::new(
                on.translation.x,
                on.translation.y - size.y - CELL_LENGTH,
                2.,
            );
            sprite.custom_size = Some(Vec2::new(size.x, CELL_LENGTH));

            // tabs
            let index_for_tab = sprites.as_ref().map(|sprites| sprites.tab).unwrap_or(0);
            for (index, tab) in tabs.into_iter().enumerate() {
                let (mut transform, mut sprite) = transforms.get_mut(tab).unwrap();
                transform.translation = Vec3::new(
                    on.translation.x + size.x * 0.5 + CELL_LENGTH * 0.5,
                    on.translation.y - CELL_LENGTH * (index as f32 + 1.),
                    1.,
                );
                sprite.custom_size = Some(Vec2::splat(CELL_LENGTH));

                if index == index_for_tab {
                    sprite.custom_size.as_mut().unwrap().x += CELL_LENGTH * 0.5;
                    transform.translation.x += CELL_LENGTH * 0.25;
                }
            }

            // tiles
            let margin = 10.;
            let space_needed_for_single_tile = margin + PICKER_TILE_LENGTH;

            let top_left_corner = on.translation + Vec2::new(size.x * -0.5, CELL_LENGTH * -0.5);
            let mut width_remaining = size.x;
            let mut height_remaining = size.y - space_needed_for_single_tile;

            for entity in not_sure.iter().flatten().copied() {
                *visibility.get_mut(entity).unwrap() = Visibility::Hidden;
            }

            let mut tiles_to_pick_iter = not_sure[index_for_tab].iter();

            for entity in tiles_to_pick_iter.by_ref() {
                if width_remaining < space_needed_for_single_tile {
                    if height_remaining < space_needed_for_single_tile {
                        break;
                    } else {
                        width_remaining = size.x;
                        height_remaining -= space_needed_for_single_tile;
                    }
                }

                width_remaining -= space_needed_for_single_tile;

                transforms.get_mut(*entity).unwrap().0.translation = Vec3::new(
                    top_left_corner.x + (size.x - width_remaining) - PICKER_TILE_LENGTH * 0.5,
                    top_left_corner.y - (size.y - height_remaining) + PICKER_TILE_LENGTH * 0.5,
                    3.,
                );
                *visibility.get_mut(*entity).unwrap() = Visibility::Visible;
            }
        },
    );

    commands
        .entity(grab)
        .observe(|on: On<WorldDrag>, mut commands: Commands| {
            commands.trigger(PickerUpdate {
                translation: on.translation,
                resize_delta: Vec2::ZERO,
            });
        })
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
        translation: Vec2::new(
            window.width() * -0.5 + size.x * 0.5,
            window.height() * 0.5 - CELL_LENGTH * 0.5,
        ),
        resize_delta: size,
    });
}

#[derive(Resource, Default)]
struct HoveredPickerSprite(Option<Entity>);

fn picker_sprite_over(
    on: On<Pointer<Over>>,
    mut hovered_picker_sprite: ResMut<HoveredPickerSprite>,
) {
    hovered_picker_sprite.0 = Some(on.entity);
}

fn picker_sprite_out(on: On<Pointer<Out>>, mut hovered_picker_sprite: ResMut<HoveredPickerSprite>) {
    if let Some(entity) = hovered_picker_sprite.0
        && entity == on.entity
    {
        hovered_picker_sprite.0 = None;
    }
}

fn picker_sprite_hover_gizmo(
    hovered_picker_sprite: Res<HoveredPickerSprite>,
    transforms: Query<&Transform>,
    mut gizmos: Gizmos,
) {
    let Some(entity) = hovered_picker_sprite.0 else {
        return;
    };

    let translation = transforms.get(entity).unwrap().translation.xy();
    gizmos.rect_2d(translation, Vec2::splat(PICKER_TILE_LENGTH), RED);
}

fn draw_selected(
    sprites: Res<Sprites>,
    transforms: Query<(&Transform, &Visibility)>,
    mut gizmos: Gizmos,
) {
    let (transform, visibility) = transforms
        .get(sprites.tabs[sprites.tab][sprites.selected].2)
        .unwrap();
    if matches!(visibility, Visibility::Hidden) {
        return;
    }

    let translation = transform.translation.xy();

    gizmos.rect_2d(translation, Vec2::splat(PICKER_TILE_LENGTH), RED);
    gizmos.rect_2d(translation, Vec2::splat(PICKER_TILE_LENGTH + 5.), YELLOW);
    gizmos.rect_2d(translation, Vec2::splat(PICKER_TILE_LENGTH + 10.), PURPLE);
}

#[derive(Resource)]
struct DialogueFont(Handle<Font>);

struct AfterDialogueAction<'w, 's, 'a>(
    &'a mut GameState,
    &'a mut HashMap<IVec2, Entity>,
    &'a mut Commands<'w, 's>,
);

#[derive(Resource, Default)]
// (messages, index of next message, run after)
struct Dialogue(Option<(Vec<String>, usize, fn(AfterDialogueAction))>);

impl Dialogue {
    fn set(
        &mut self,
        messages: impl IntoIterator<Item = impl Into<String>>,
        run_after: fn(AfterDialogueAction),
    ) {
        self.0 = Some((
            messages.into_iter().map(|message| message.into()).collect(),
            0,
            run_after,
        ));
    }
}

#[derive(Event)]
struct NextDialogue;

fn display_dialogue(dialogue: Res<Dialogue>, mut commands: Commands) {
    let Some(dialogue) = dialogue.0.as_ref() else {
        return;
    };

    if dialogue.1 == 0 {
        commands.trigger(NextDialogue);
    }
}

fn next_dialogue(
    _: On<NextDialogue>,
    font: Res<DialogueFont>,
    mut dialogue_box: Single<(Entity, &mut Visibility, &mut UiHover), With<DialogueBox>>,
    mut maybe_dialogue: ResMut<Dialogue>,
    mut state: ResMut<GameState>,
    mut commands: Commands,
    mut tiles: ResMut<Tiles>,
) {
    let Some(dialogue) = maybe_dialogue.0.as_mut() else {
        return;
    };
    info!("Next dialogue.");

    commands.entity(dialogue_box.0).despawn_children();
    *dialogue_box.1 = Visibility::Visible;

    if dialogue.1 == dialogue.0.len() {
        dialogue.2(AfterDialogueAction(&mut state, &mut tiles.0, &mut commands));
        maybe_dialogue.0 = None;
        *dialogue_box.1 = Visibility::Hidden;
        *dialogue_box.2 = UiHover(false);
        return;
    }

    for (index, line) in dialogue.0[dialogue.1].lines().enumerate() {
        let line = if index == 0 {
            format!("* {line}")
        } else {
            format!("   {line}")
        };

        let dialogue_box_line = commands
            .spawn(Node {
                top: px(40),
                left: px(40),
                ..default()
            })
            .id();
        commands.entity(dialogue_box.0).add_child(dialogue_box_line);
        let mut dialogue_box_line = commands.entity(dialogue_box_line);

        for char in line.chars() {
            dialogue_box_line.with_child((
                Text::new(char),
                TextFont {
                    font: font.0.clone(),
                    font_size: 40.,
                    ..default()
                },
            ));
            dialogue_box_line.with_child((
                Text::new(" "),
                TextFont {
                    font: font.0.clone(),
                    font_size: 15.,
                    ..default()
                },
            ));
        }
    }

    dialogue.1 += 1;
}

#[derive(Component)]
struct DialogueBox;

#[derive(Component)]
struct DoneButton;

#[derive(Component)]
struct ModeButton;

#[derive(Resource, Default)]
enum Mode {
    #[default]
    Place,
    Erase,
}

fn spawn_dialogue_box(commands: &mut Commands, asset_server: &AssetServer) {
    const SCALE: f32 = 1.5;

    let font = asset_server.load("8bitoperator_jve.ttf");
    let cloned_font = font.clone();
    commands.insert_resource(DialogueFont(font));

    commands
        .spawn((
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::End,
                justify_content: JustifyContent::End,
                ..default()
            },
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    UiHover(false),
                    UiDragged(false),
                    Visibility::Hidden,
                    DoneButton,
                    Node {
                        margin: px(5).all(),
                        width: px(150),
                        height: px(70),
                        border: px(5).all(),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BorderColor::all(WHITE),
                    BackgroundColor(GREEN.into()),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new("DONE"),
                        TextFont {
                            font: cloned_font.clone(),
                            font_size: 40.,
                            ..default()
                        },
                        TextColor(WHITE.into()),
                    ));
                })
                .observe(|on: On<Pointer<Over>>, mut hover: Query<&mut UiHover>| {
                    hover.get_mut(on.entity).unwrap().0 = true;
                })
                .observe(|on: On<Pointer<Out>>, mut hover: Query<&mut UiHover>| {
                    hover.get_mut(on.entity).unwrap().0 = false;
                })
                .observe(
                    |on: On<Pointer<Click>>,
                     mut button: Query<(&mut Visibility, &mut UiHover)>,
                     mut state: ResMut<GameState>| {
                        let mut button = button.get_mut(on.entity).unwrap();

                        *button.0 = Visibility::Hidden;
                        button.1.0 = false;

                        let GameState::Build(build) = &mut *state else {
                            error!("Somehow the DONE button was pressed in the wrong state!");
                            return;
                        };

                        build.done = true;
                    },
                );
        });

    commands
        .spawn((
            Pickable {
                should_block_lower: false,
                is_hoverable: false,
            },
            Node {
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::End,
                justify_content: JustifyContent::Start,
                ..default()
            },
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    UiHover(false),
                    UiDragged(false),
                    Visibility::Hidden,
                    ModeButton,
                    Node {
                        margin: px(5).all(),
                        width: px(150),
                        height: px(70),
                        border: px(5).all(),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BorderColor::all(WHITE),
                    BackgroundColor(PURPLE.into()),
                ))
                .with_children(|builder| {
                    builder.spawn((
                        Text::new("PLACE"),
                        TextFont {
                            font: cloned_font.clone(),
                            font_size: 40.,
                            ..default()
                        },
                        TextColor(WHITE.into()),
                    ));
                })
                .observe(|on: On<Pointer<Over>>, mut hover: Query<&mut UiHover>| {
                    hover.get_mut(on.entity).unwrap().0 = true;
                })
                .observe(|on: On<Pointer<Out>>, mut hover: Query<&mut UiHover>| {
                    hover.get_mut(on.entity).unwrap().0 = false;
                })
                .observe(
                    move |on: On<Pointer<Click>>,
                          mut background_colour: Query<&mut BackgroundColor>,
                          mut commands: Commands,
                          mut mode: ResMut<Mode>| {
                        let mut background_colour = background_colour.get_mut(on.entity).unwrap();

                        let mut entity = commands.entity(on.entity);
                        entity.despawn_children();
                        match *mode {
                            Mode::Erase => {
                                *mode = Mode::Place;
                                *background_colour = BackgroundColor(PURPLE.into());
                                entity.with_child((
                                    Text::new("PLACE"),
                                    TextFont {
                                        font: cloned_font.clone(),
                                        font_size: 40.,
                                        ..default()
                                    },
                                    TextColor(WHITE.into()),
                                ));
                            }
                            Mode::Place => {
                                *mode = Mode::Erase;
                                *background_colour = BackgroundColor(BLACK.into());
                                entity.with_child((
                                    Text::new("ERASE"),
                                    TextFont {
                                        font: cloned_font.clone(),
                                        font_size: 40.,
                                        ..default()
                                    },
                                    TextColor(WHITE.into()),
                                ));
                            }
                        }
                    },
                );
        });

    commands
        .spawn((
            Visibility::Hidden,
            Interaction::None,
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
            UiHover(false),
            UiDragged(false),
            DialogueBox,
            Node {
                align_items: AlignItems::Start,
                justify_content: JustifyContent::Start,
                width: px(593. * SCALE),
                height: px(167. * SCALE),
                flex_direction: FlexDirection::Column,
                justify_self: JustifySelf::End,
                ..default()
            },
            ImageNode {
                image: asset_server.load("box.png"),
                image_mode: NodeImageMode::Auto,
                ..default()
            },
        ))
        .observe(|on: On<Pointer<Over>>, mut hover: Query<&mut UiHover>| {
            hover.get_mut(on.entity).unwrap().0 = true;
        })
        .observe(|on: On<Pointer<Out>>, mut hover: Query<&mut UiHover>| {
            hover.get_mut(on.entity).unwrap().0 = false;
        })
        .observe(|_: On<Pointer<Click>>, mut commands: Commands| {
            commands.trigger(NextDialogue);
        });
}

struct Build {
    seconds_spent: f32,

    done: bool,

    dialogue_seconds_remaining: f32,

    blueprint_seconds_remaining: f32,
    blue_print_entities: Vec<Entity>,

    tiles: BuildTiles,
}

#[derive(Resource, Default)]
enum GameState {
    #[default]
    Paused,
    Rest(f32),
    Build(Build),
}

impl GameState {
    const REST: Self = GameState::Rest(2.);
}

fn game_state(
    mut state: ResMut<GameState>,
    time: Res<Time>,
    mut dialogue: ResMut<Dialogue>,
    mut commands: Commands,
    sprites: Res<Sprites>,
    mut visibility: Query<&mut Visibility, Without<DoneButton>>,
    mut done: Single<&mut Visibility, With<DoneButton>>,
    tiles: Res<Tiles>,
    sprite_markers: Query<&SpriteMarker>,
) {
    match &mut *state {
        GameState::Paused => (),
        GameState::Rest(seconds_remaining) => {
            *seconds_remaining -= time.delta_secs();

            if *seconds_remaining < 0. {
                let build_tiles = BUILD_TILES[rand::random_range(0..BUILD_TILES.len())];
                dialogue.set([format!("MIKE, {}, please!", build_tiles.1)], |_| {});
                **done = Visibility::Visible;
                *state = GameState::Build(Build {
                    seconds_spent: 0.,

                    done: false,

                    dialogue_seconds_remaining: 5.,

                    blueprint_seconds_remaining: build_tiles.2,
                    blue_print_entities: build_tiles
                        .0
                        .iter()
                        .map(|(x, y, tab, selected)| {
                            let selected = sprites.tabs[*tab][*selected];

                            let mut sprite = Sprite::from_atlas_image(
                                sprites.texture.clone(),
                                TextureAtlas {
                                    layout: sprites.texture_atlas_layouts[selected.0].clone(),
                                    index: selected.1,
                                },
                            );
                            sprite.custom_size = Some(Vec2::splat(CELL_LENGTH));
                            sprite.flip_x = selected.3;
                            sprite.color = Color::srgba(0.8, 0.8, 1., 0.3);

                            commands
                                .spawn((
                                    Transform::from_translation(
                                        grid_to_world(IVec2::new(*x, *y)).extend(-1.),
                                    ),
                                    sprite,
                                ))
                                .id()
                        })
                        .collect(),

                    tiles: build_tiles.0,
                });
            }
        }
        GameState::Build(build) => {
            let mut clear_dialogue = || {
                let empty: [&'static str; 0] = [];
                dialogue.set(empty, |_| {});
            };
            let mut clear_blueprints = || {
                for entity in build.blue_print_entities.iter().copied() {
                    *visibility.get_mut(entity).unwrap() = Visibility::Hidden;
                }
            };

            let time_delta_seconds = time.delta_secs();
            build.seconds_spent += time_delta_seconds;

            if build.done {
                //clear_dialogue();
                clear_blueprints();

                info!("{}", build.seconds_spent);
                let mut time_taken_string = String::new();

                let all_seconds = build.seconds_spent as u32;
                let seconds = all_seconds % 60;
                let all_minutes = all_seconds / 60;

                if all_minutes != 0 {
                    time_taken_string.push_str(&format!("{all_minutes} minutes, and "));
                }
                time_taken_string.push_str(&format!("{seconds} seconds"));

                let mut tiles_placed: Vec<(i32, i32, usize, usize)> = tiles
                    .0
                    .iter()
                    .map(|(translation, entity)| {
                        let sprite_marker = sprite_markers.get(*entity).unwrap();
                        (
                            translation.x,
                            translation.y,
                            sprite_marker.0,
                            sprite_marker.1,
                        )
                    })
                    .collect();

                let mut missing_tiles = 0;
                let mut wrong_tiles = 0;
                let mut correct_tiles = 0;
                for (x, y, tab, selected) in build.tiles {
                    if let Some((index, placed_tab, placed_selected)) =
                        tiles_placed.iter().enumerate().find_map(
                            |(index, (placed_x, placed_y, placed_tab, placed_selected))| {
                                if x == placed_x && y == placed_y {
                                    Some((index, placed_tab, placed_selected))
                                } else {
                                    None
                                }
                            },
                        )
                    {
                        if tab == placed_tab && selected == placed_selected {
                            correct_tiles += 1;
                        } else {
                            wrong_tiles += 1;
                        }
                        // Save a little bit of time for future searches. Can't have two tiles in the same place.
                        // It also allows us to calculate unnecessary tiles.
                        tiles_placed.swap_remove(index);
                    } else {
                        missing_tiles += 1;
                    }
                }
                let percent_correct =
                    ((correct_tiles as f32 / build.tiles.len() as f32) * 100.).round() as u32;
                let unnecessary = tiles_placed.len();

                dialogue.set([format!("Thank you Mike!\nYou took {time_taken_string}."), format!("You were {percent_correct}% correct.\nYou used {wrong_tiles} wrong tiles.\nYou missed {missing_tiles} tiles.\nYou placed {unnecessary} unnecessary tiles.")], |state| {
                    *state.0 = GameState::REST;
                    for (_, entity) in state.1.drain() {
                        state.2.entity(entity).despawn();
                    }
                });

                *state = GameState::Paused;
                return;
            }

            if build.dialogue_seconds_remaining >= 0. {
                build.dialogue_seconds_remaining -= time_delta_seconds;

                if build.dialogue_seconds_remaining < 0. {
                    clear_dialogue();
                }
            }

            if build.blueprint_seconds_remaining >= 0. {
                build.blueprint_seconds_remaining -= time_delta_seconds;

                if build.blueprint_seconds_remaining < 0. {
                    clear_blueprints()
                }
            }
        }
    }
}

type BuildTiles = &'static [(i32, i32, usize, usize)];

const BUILD_TILES: &[(BuildTiles, &str, f32)] = &[
    (STATUE, "build me a statue", 10.),
    (SHELTER, "construct the shelter", 30.),
];

const STATUE: BuildTiles = &[
    (4, 2, 1, 180),
    (6, 2, 1, 182),
    (4, 1, 1, 200),
    (4, 3, 1, 160),
    (5, 2, 1, 181),
    (6, 1, 1, 202),
    (4, 0, 1, 220),
    (6, 0, 1, 222),
    (6, 3, 1, 162),
    (5, 0, 1, 221),
    (5, 1, 1, 201),
    (5, 3, 1, 161),
];

const SHELTER: BuildTiles = &[
    (1, -5, 1, 229),
    (3, -4, 1, 229),
    (-2, -3, 1, 261),
    (-1, -3, 1, 229),
    (1, -3, 1, 229),
    (-2, -2, 1, 261),
    (-1, -1, 1, 229),
    (3, -6, 1, 263),
    (4, -3, 1, 209),
    (9, -3, 1, 279),
    (-1, -5, 1, 230),
    (0, -5, 1, 229),
    (3, -5, 1, 229),
    (-2, 0, 1, 261),
    (-1, -2, 1, 229),
    (0, -3, 1, 229),
    (4, -5, 1, 229),
    (7, -5, 1, 229),
    (4, -2, 1, 189),
    (5, -5, 1, 229),
    (7, -2, 1, 229),
    (3, 0, 1, 229),
    (-2, -1, 1, 261),
    (-2, -6, 1, 260),
    (7, -6, 1, 264),
    (8, -1, 1, 230),
    (2, -3, 1, 207),
    (6, -3, 1, 229),
    (9, -6, 1, 259),
    (7, -3, 1, 229),
    (4, 1, 1, 229),
    (9, -5, 1, 279),
    (2, -5, 1, 229),
    (-1, 1, 1, 270),
    (1, 0, 1, 229),
    (0, 1, 1, 271),
    (0, -2, 1, 229),
    (6, -4, 1, 229),
    (9, -2, 1, 279),
    (3, 1, 1, 229),
    (0, 0, 1, 229),
    (5, -4, 1, 229),
    (5, -2, 1, 190),
    (7, -4, 1, 230),
    (4, -4, 1, 229),
    (-1, -4, 1, 229),
    (6, -5, 1, 229),
    (9, -4, 1, 279),
    (-1, 0, 1, 229),
    (8, -5, 1, 229),
    (5, -1, 1, 170),
    (6, -6, 1, 263),
    (1, 1, 1, 272),
    (7, -1, 1, 229),
    (1, -2, 1, 229),
    (2, -4, 1, 229),
    (2, -1, 1, 167),
    (6, 1, 1, 277),
    (8, -6, 1, 262),
    (6, 0, 1, 229),
    (7, 0, 1, 229),
    (1, -6, 1, 264),
    (3, -1, 1, 168),
    (0, -1, 1, 230),
    (-2, -5, 1, 261),
    (0, -4, 1, 229),
    (3, -3, 1, 208),
    (2, -6, 1, 262),
    (8, -2, 1, 229),
    (-1, -6, 1, 262),
    (9, 0, 1, 279),
    (8, 1, 1, 275),
    (6, -1, 1, 229),
    (1, -4, 1, 229),
    (4, -6, 1, 264),
    (-2, -4, 1, 261),
    (7, 1, 1, 276),
    (4, 0, 1, 229),
    (1, -1, 1, 229),
    (4, -1, 1, 169),
    (5, 0, 1, 229),
    (6, -2, 1, 229),
    (9, -1, 1, 279),
    (3, -2, 1, 188),
    (2, 1, 1, 273),
    (2, 0, 1, 229),
    (8, 0, 1, 229),
    (5, -3, 1, 210),
    (8, -4, 1, 229),
    (2, -2, 1, 187),
    (-2, 1, 1, 269),
    (8, -3, 1, 229),
    (0, -6, 1, 263),
    (9, 1, 1, 274),
    (5, -6, 1, 262),
    (5, 1, 1, 278),
];
