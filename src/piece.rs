use std::fmt::{Debug, Display};
use std::ptr::null;
use bevy::prelude::*;
use bevy::prelude::Color::Rgba;

use crate::board::{BoardResource, SQUARE_SIZE, square_to_vector, WorldCursor};
use crate::logic::{Coordinate, Piece, PieceColor, PieceKind};

#[derive(Component)]
pub struct ShadowPiece {}

#[derive(Component)]
pub struct PhantomPiece {}

#[derive(Component)]
pub struct PieceComponent {
    piece: Piece,
    dragged: bool
}

impl PieceComponent {
    fn get_texture_name(&self) -> String {
        format!("{}_{}", self.piece.color.to_string(), self.piece.kind.to_string())
    }
}

#[derive(Event)]
pub struct BoardUpdate {}

pub fn update_board_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut replace_event_listener: EventReader<BoardUpdate>,
    mut pieces_query: Query<Entity, (With<PieceComponent>, Without<PromotionOption>)>,
    board: Res<BoardResource>
) {
    for _ in replace_event_listener.read() {
        for entity in pieces_query.iter() {
            commands.entity(entity).despawn();
        }
        for (square, piece) in board.0.pieces.iter() {
            let piece_component = PieceComponent{piece: piece.clone(), dragged: false};
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(SQUARE_SIZE * 0.9, SQUARE_SIZE * 0.9)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::from((square_to_vector(square.clone()), 1.0))),
                    texture: asset_server.load(piece_component.get_texture_name() + ".png"),
                    ..default()
                }, piece_component)
            );
        }
        break;
    }
}

#[derive(Resource)]
pub struct AllowDrag(pub bool);

#[derive(Component)]
pub struct PromotionOption;

#[derive(Resource)]
pub struct PromotionSquare(pub Option<Coordinate>);
pub fn promotion_chooser(
    mut board: ResMut<BoardResource>,
    mut allow_drag: ResMut<AllowDrag>,
    mut promotion_square: ResMut<PromotionSquare>,
    cursor_query: Option<Res<WorldCursor>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut board_update_writer: EventWriter<BoardUpdate>,
    mut promotion_options: Query<(&mut Transform, &mut Visibility, &PieceComponent), With<PromotionOption>>
) {
    if let Some(square) = promotion_square.0 {
        let Some(cursor) = cursor_query else { return };
        if cursor.square != square { return };
        if !mouse_button.just_pressed(MouseButton::Left) { return };

        let mut min_distance = f32::MAX;
        let mut min_piece = None;
        for (transform, visibility, sprite) in promotion_options.iter() {
            if visibility == Visibility::Hidden { continue };
            let distance = transform.translation.truncate().distance(cursor.position);
            if distance < min_distance {
                min_distance = distance;
                min_piece = Some(sprite.piece);
            }
        }
        for (_, mut visibility, _) in promotion_options.iter_mut() {
            *visibility = Visibility::Hidden;
        }
        board.0.pieces.insert(square, Piece{kind: min_piece.unwrap().kind, color: min_piece.unwrap().color, square, moved: false});
        allow_drag.0 = true;
        promotion_square.0 = None;
        board_update_writer.send(BoardUpdate{});

    } else {
        for rank in [7i8, 0] {
            for file in 0i8..=7 {
                let position = Coordinate(file, rank);
                let Some(occupying) = board.0.pieces.get(&position) else { continue };
                if occupying.kind == PieceKind::PAWN {
                    allow_drag.0 = false;
                    promotion_square.0 = Some(position);

                    for (mut transform, mut visibility, sprite) in promotion_options.iter_mut() {
                        if sprite.piece.color != occupying.color { continue };
                        transform.translation = Vec3::from((square_to_vector(position), 21.37));
                        match sprite.piece.kind {
                            PieceKind::QUEEN => {
                                transform.translation.x -= SQUARE_SIZE / 4.0;
                                transform.translation.y += SQUARE_SIZE / 4.0;
                            }
                            PieceKind::ROOK => {
                                transform.translation.x += SQUARE_SIZE / 4.0;
                                transform.translation.y += SQUARE_SIZE / 4.0;
                            }
                            PieceKind::BISHOP => {
                                transform.translation.x -= SQUARE_SIZE / 4.0;
                                transform.translation.y -= SQUARE_SIZE / 4.0;
                            }
                            PieceKind::KNIGHT => {
                                transform.translation.x += SQUARE_SIZE / 4.0;
                                transform.translation.y -= SQUARE_SIZE / 4.0;
                            }
                            _ => {}
                        }
                        *visibility = Visibility::Visible;
                    }

                    board.0.pieces.remove(&position);
                    board_update_writer.send(BoardUpdate{});
                    break;
                }
            }
        }
    }
}
#[derive(Resource)]
pub struct CheckAnimationTimer(pub Timer);
pub fn check_animation(
    time: Res<Time>,
    mut animation_timer: ResMut<CheckAnimationTimer>,
    board: Res<BoardResource>,
    mut sprite_pieces: Query<(&mut Sprite, &PieceComponent), (Without<ShadowPiece>, Without<PhantomPiece>, Without<PromotionOption>)>,
) {
    for (mut sprite, piece_component) in sprite_pieces.iter_mut() {
        if piece_component.piece.kind != PieceKind::KING { continue };
        if !board.0.is_checked(&piece_component.piece) { continue };
        if !board.0.has_moves(piece_component.piece.color) {
            sprite.color.set_a(0.5);
            return;
        }
        animation_timer.0.tick(time.delta());
        if animation_timer.0.just_finished() {
            if sprite.color.a() == 1.0 {
                sprite.color.set_a(0.75);
            } else {
                sprite.color.set_a(1.0);
            }
        }
        return;
    }
    animation_timer.0.reset();
}
pub fn drag_piece(
    mouse_button: Res<ButtonInput<MouseButton>>,
    cursor_query: Option<Res<WorldCursor>>,
    allow_drag: Res<AllowDrag>,
    mut shadow_query: Query<(&mut Visibility, &mut Transform, &mut Handle<Image>), With<ShadowPiece>>,
    mut phantom_query: Query<(&mut Visibility, &mut Transform, &mut Handle<Image>), (With<PhantomPiece>, Without<ShadowPiece>)>,
    mut sprite_pieces: Query<(&mut PieceComponent, &mut Transform, &Handle<Image>), (Without<ShadowPiece>, Without<PhantomPiece>, Without<PromotionOption>)>,
    mut board: ResMut<BoardResource>,
    mut board_update_writer: EventWriter<BoardUpdate>
) {
    if (!allow_drag.0) { return };
    let Some(cursor) = cursor_query else { return };

    let (mut shadow_visibility, mut shadow_transform, mut shadow_texture) = shadow_query.single_mut();
    let (mut phantom_visibility, mut phantom_transform, mut phantom_texture) = phantom_query.single_mut();

    for (mut sprite, mut transform, texture) in sprite_pieces.iter_mut() {
        if sprite.piece.color != board.0.on_move { continue };
        if sprite.piece.square == cursor.square && mouse_button.just_pressed(MouseButton::Left) {
            sprite.dragged = true;
            *shadow_texture = texture.clone();
            *phantom_texture = texture.clone();
            phantom_transform.translation = Vec3::from((square_to_vector(cursor.square), 1.0));
            *phantom_visibility = Visibility::Visible;
        }
        if !sprite.dragged { continue };
        let can_move = board.0.get_valid_moves(board.0.pieces.get(&sprite.piece.square).unwrap()).contains(&cursor.square);

        if mouse_button.just_released(MouseButton::Left) {
            sprite.dragged = false;
            *shadow_visibility = Visibility::Hidden;
            *phantom_visibility = Visibility::Hidden;
            if can_move {
                board.0.move_piece(&sprite.piece.square, &cursor.square);
                board.0.flip_on_move();
                board_update_writer.send(BoardUpdate{});
            }
            transform.translation = Vec3::from((square_to_vector(sprite.piece.square), 1.0));

            return;
        }
        transform.translation = Vec3::from((cursor.position, 10.0));
        shadow_transform.translation = Vec3::from((cursor.square_center, 2.0));
        *shadow_visibility = if can_move { Visibility::Visible } else { Visibility::Hidden };
        return;
    }

}
pub fn spawn_phantom_piece(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(SQUARE_SIZE * 0.9, SQUARE_SIZE * 0.9)),
                color: Rgba {red: 1.0, green: 1.0, blue: 1.0, alpha: 0.5},
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        }, ShadowPiece{})
    );
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(SQUARE_SIZE * 0.9, SQUARE_SIZE * 0.9)),
                color: Rgba {red: 1.0, green: 1.0, blue: 1.0, alpha: 0.5},
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        }, PhantomPiece{})
    );
}

pub fn spawn_promotion_options(mut commands: Commands, asset_server: Res<AssetServer>) {
    for color in [PieceColor::WHITE, PieceColor::BLACK] {
        for piece_kind in [PieceKind::QUEEN, PieceKind::ROOK, PieceKind::BISHOP, PieceKind::KNIGHT] {
            let piece = PieceComponent { piece: Piece { kind: piece_kind, color, square: Coordinate(5, 5), moved: false }, dragged: false };
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(SQUARE_SIZE * 0.9 * 0.5, SQUARE_SIZE * 0.9 * 0.5)),
                        color: Rgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 },
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    texture: asset_server.load(piece.get_texture_name() + ".png"),
                    ..default()
                }, PromotionOption {}, piece)
            );
        }
    }
}