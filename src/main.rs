mod piece;
mod board;
mod logic;

use std::time::Duration;
use bevy::app::{App, Startup};
use bevy::DefaultPlugins;
use bevy::prelude::*;
use crate::board::{spawn_board, SQUARE_SIZE, update_board_cursor, update_outline};
use crate::piece::{BoardUpdate, drag_piece, spawn_phantom_piece, update_board_pieces, AllowDrag, promotion_chooser, spawn_promotion_options, PromotionSquare, check_animation, CheckAnimationTimer};

fn main() {
    App::new()
        .insert_resource(AllowDrag(true))
        .insert_resource(PromotionSquare(None))
        .insert_resource(CheckAnimationTimer(Timer::new(Duration::from_millis(500), TimerMode::Repeating)))
        .add_event::<BoardUpdate>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (spawn_camera, spawn_board, spawn_phantom_piece, spawn_promotion_options))
        .add_systems(Update, ((update_board_cursor, drag_piece, promotion_chooser, update_board_pieces).chain(), check_animation, update_outline))
        .run();
}


pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(
        Camera2dBundle {
            transform: Transform::from_xyz(SQUARE_SIZE * 3.5, SQUARE_SIZE * 3.5, 0.0),
            ..default()
        }
    );
}