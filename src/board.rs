use bevy::math::Vec2;
use bevy::prelude::{Camera, Color, Commands, Component, default, EventWriter, GlobalTransform, Query, Res, Resource, Sprite, SpriteBundle, Transform, Window, With};
use bevy::window::PrimaryWindow;
use crate::logic::{Board, Coordinate, PieceColor};
use crate::piece::BoardUpdate;

pub const SQUARE_SIZE: f32 = 64.0;

#[derive(Component)]
pub struct BoardTile {
    pub square: (i8, i8)
}

impl BoardTile {
    pub fn get_color(&self) -> Color {
        match (self.square.0 + self.square.1) % 2 {
            0 => Color::WHITE,
            1 => Color::BLACK,
            _ => unreachable!()
        }
    }
}
#[derive(Resource)]
pub struct BoardResource(pub Board);

#[derive(Component)]
pub struct BoardOutline;

pub fn spawn_board(mut commands: Commands, mut board_update_writer: EventWriter<BoardUpdate>) {
    commands.insert_resource(BoardResource(Board::new()));
    for col in 0..8i8 {
        for row in 0..8i8 {
            let tile = BoardTile{square: (col, row)};
            commands.spawn((SpriteBundle{
                transform: Transform::from_xyz(
                    f32::from(tile.square.0) * SQUARE_SIZE,
                    f32::from(tile.square.1) * SQUARE_SIZE,
                    0.0
                ),
                sprite: Sprite {
                    color: tile.get_color(),
                    custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                    ..default()
                },
                ..default()
            }, tile));
        }
    }
    commands.spawn((SpriteBundle{
        transform: Transform::from_xyz(
            SQUARE_SIZE * 3.5,
            SQUARE_SIZE * 3.5,
            -1.0
        ),
        sprite: Sprite {
            color: Color::WHITE,
            custom_size: Some(Vec2::new(SQUARE_SIZE*9.0, SQUARE_SIZE*9.0)),
            ..default()
        },
        ..default()
    }, BoardOutline));
    board_update_writer.send(BoardUpdate{});
}
pub fn update_outline(board: Res<BoardResource>, mut outline_query: Query<&mut Sprite, With<BoardOutline>>) {
    let mut outline = outline_query.single_mut();
    if board.0.has_moves(board.0.on_move) {
        outline.color = if board.0.on_move == PieceColor::WHITE { Color::WHITE } else { Color::BLACK };
    } else {
        outline.color = Color::GRAY;
    }
}
fn vector_to_square(vec: Vec2) -> Coordinate {
    Coordinate((vec.x / SQUARE_SIZE + 0.5) as i8, (vec.y / SQUARE_SIZE + 0.5) as i8)
}

pub fn square_to_vector(square: Coordinate) -> Vec2 {
    Vec2::new(square.0 as f32 * SQUARE_SIZE, square.1 as f32 * SQUARE_SIZE)
}
#[derive(Resource)]
pub struct WorldCursor {
    pub position: Vec2,
    pub square: Coordinate,
    pub square_center: Vec2
}

impl WorldCursor {
    fn from_position(position: Vec2) -> Self {
        let square = vector_to_square(position);
        WorldCursor {position, square, square_center: square_to_vector(square)}
    }
}

pub fn update_board_cursor(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands
) {
    let (camera, camera_transform) = camera_query.single();
    let position = window_query.single().cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor)
            .map(|ray| ray.origin.truncate()));
    let Some(cursor_position) = position else { commands.remove_resource::<WorldCursor>(); return };
    commands.insert_resource(WorldCursor::from_position(cursor_position));
}

