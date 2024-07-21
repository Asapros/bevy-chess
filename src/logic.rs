use std::collections::HashMap;
use std::fmt::Display;
use std::iter::{IntoIterator, Iterator};

#[derive(Copy, Clone, PartialEq)]
pub enum PieceKind {
    PAWN,
    ROOK,
    KNIGHT,
    BISHOP,
    KING,
    QUEEN
}

impl Display for PieceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            PieceKind::PAWN => "pawn",
            PieceKind::ROOK => "rook",
            PieceKind::KNIGHT => "knight",
            PieceKind::BISHOP => "bishop",
            PieceKind::KING => "king",
            PieceKind::QUEEN => "queen"
        })
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum PieceColor {
    WHITE,
    BLACK
}

impl Display for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            PieceColor::WHITE => "white",
            PieceColor::BLACK => "black"
        })
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct Coordinate(pub i8, pub i8);

#[derive(Copy, Clone)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: PieceColor,
    pub square: Coordinate,
    pub moved: bool
}

#[derive(Clone)]
pub struct Board {
    pub pieces: HashMap<Coordinate, Piece>,
    pub on_move: PieceColor,
    pub turn_number: u32,
    pub en_pessant_file: Option<i8>
}
const ROOK_PATTERN: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const BISHOP_PATTERN: [(i8, i8); 4] = [(1, 1), (-1, 1), (1, -1), (-1, -1)];
const KNIGHT_PATTERN: [(i8, i8); 8] = [(1, 2), (2, 1), (-1, 2), (2, -1), (1, -2), (-2, 1), (-1, -2), (-2, -1)];

impl Board {
    pub fn new() -> Self {
        let mut starting: HashMap<Coordinate, Piece> = HashMap::new();
        for color in [PieceColor::WHITE, PieceColor::BLACK] {
            for (index, kind) in [PieceKind::ROOK, PieceKind::KNIGHT, PieceKind::BISHOP, PieceKind::KING, PieceKind::QUEEN, PieceKind::BISHOP, PieceKind::KNIGHT, PieceKind::ROOK].iter().enumerate() {
                let row = if color == PieceColor::WHITE { 0i8 } else { 7i8 };
                let coordinate = Coordinate(index as i8, row);
                starting.insert( coordinate, Piece{kind: kind.clone(), color, square: coordinate, moved: false});
            }
            let row = if color == PieceColor::WHITE { 1i8 } else { 6i8 };
            for col in 0..=7i8 {
                let coordinate = Coordinate(col, row);
                starting.insert(coordinate, Piece{kind: PieceKind::PAWN, color, square: coordinate, moved: false});
            }
        }
        Board {pieces: starting, on_move: PieceColor::WHITE, turn_number: 0, en_pessant_file: None}
    }

    pub fn has_moves(&self, color: PieceColor) -> bool {
        for (_, piece) in self.pieces.iter() {
            if piece.color != color { continue };
            if !self.get_valid_moves(&piece).is_empty() { return true };
        }
        return false;
    }

    pub fn looking_at(&self, piece: &Piece) -> Vec<Coordinate> {
        let mut look = Vec::new();
        match piece.kind {
            PieceKind::KING => {
                for x in -1..=1i8 {
                    for y in -1..=1i8 {
                        if x == 0 && y == 0 { continue };
                        let new_square = Coordinate(piece.square.0 + x, piece.square.1 + y);
                        if new_square.0 < 0 || new_square.1 < 0 { continue };
                        if new_square.0 > 7 || new_square.1 > 7 { continue };
                        if let Some(occupying) = self.pieces.get(&new_square) {
                            if piece.color == occupying.color { continue };
                        }
                        look.push(new_square);
                    }
                }
            }
            PieceKind::ROOK | PieceKind::BISHOP | PieceKind::QUEEN => {
                let deltas: Vec<(i8, i8)> = match piece.kind {
                    PieceKind::ROOK => { ROOK_PATTERN.to_vec() },
                    PieceKind::BISHOP => { BISHOP_PATTERN.to_vec() },
                    PieceKind::QUEEN => { ROOK_PATTERN.into_iter().chain(BISHOP_PATTERN.into_iter()).collect() }
                    _ => {unreachable!()}
                };

                for delta in deltas {
                    let mut check = piece.square;
                    loop {
                        check = Coordinate((check.0 + delta.0), (check.1 + delta.1));
                        if (check.0 < 0 || check.0 > 7 || check.1 < 0 || check.1 > 7) { break; };
                        if let Some(occupying) = self.pieces.get(&check) {
                            if piece.color != occupying.color {
                                look.push(check);
                            }
                            break;
                        }
                        look.push(check);
                    }
                }
            }
            PieceKind::KNIGHT => {
                for delta in KNIGHT_PATTERN {
                    let moved = Coordinate(piece.square.0 + delta.0, piece.square.1 + delta.1);
                    if (moved.0 < 0 || moved.0 > 7 || moved.1 < 0 || moved.1 > 7) { continue; };
                    if let Some(occupying) = self.pieces.get(&moved) {
                        if piece.color == occupying.color { continue };
                    }
                    look.push(moved);
                }
            }
            PieceKind::PAWN => {
                let direction = if piece.color == PieceColor::WHITE { 1i8 } else { -1i8 };

                for capture_delta in [Coordinate(1i8, direction), Coordinate(-1, direction)] {
                    let capture_square = Coordinate(piece.square.0 + capture_delta.0, piece.square.1 + capture_delta.1);
                    if capture_square.0 < 0 || capture_square.0 > 7 { continue };
                    if let Some(occupying) = self.pieces.get(&capture_square) {
                        if (piece.color != occupying.color) {
                            look.push(capture_square);
                        }
                    }
                }

            }
            _ => {}
        }
        look

    }

    pub fn is_checked(&self, piece: &Piece) -> bool {
        for (_, checking) in self.pieces.iter() {
            if checking.color == piece.color { continue };
            if self.looking_at(&checking).contains(&piece.square) { return true };
        }
        false
    }

    pub fn get_valid_moves(&self, piece: &Piece) -> Vec<Coordinate>{
        let mut potential_moves =  self.looking_at(piece);
        if piece.kind == PieceKind::PAWN {
            let direction = if piece.color == PieceColor::WHITE { 1i8 } else { -1i8 };
            let following = Coordinate(piece.square.0, piece.square.1 + direction);
            if self.pieces.get(&following).is_none() && following.1 >= 0 && following.1 <= 7{
                potential_moves.push(following);
                let following_following = Coordinate(following.0, following.1 + direction);
                if !piece.moved && self.pieces.get(&following_following).is_none() && following_following.1 >= 0 && following_following.1 <= 7 {
                    potential_moves.push(following_following);
                }
            }
        }


        let mut moves = Vec::new();
        for potential_move in potential_moves {
            let mut potential_board = self.clone();
            potential_board.pieces.remove(&piece.square);
            let mut potential_piece = piece.clone();
            potential_piece.square = potential_move;
            potential_board.pieces.insert(potential_move, potential_piece);
            let Some(king) = potential_board.pieces.iter().find_map(|(_, potential_king)| {
                if potential_king.kind == PieceKind::KING && piece.color == potential_king.color { Some(potential_king) } else {None}}
            ) else { panic!("no king") };
            if !potential_board.is_checked(king) { moves.push(potential_move) };
        }

        if piece.kind == PieceKind::KING && !piece.moved && !self.is_checked(&piece) {
            let rank = piece.square.1;
            for direction in [-1i8, 1] {
                if !moves.contains(&Coordinate(piece.square.0+direction, rank)) { continue };
                if self.pieces.contains_key(&Coordinate(piece.square.0+direction*2, rank)) { continue };
                if self.is_checked(&Piece{kind: PieceKind::KING, color: piece.color, square: Coordinate(piece.square.0+direction*2, rank), moved: false}) { continue };

                let mut distance = 2;
                while piece.square.0 + direction*distance < 8 && piece.square.0 + direction*distance >= 0 {
                    distance += 1;
                    let Some(occupying) = self.pieces.get(&Coordinate(piece.square.0+direction*distance, rank)) else { continue };
                    if occupying.kind == PieceKind::ROOK && occupying.color == piece.color && !occupying.moved {
                        moves.push(Coordinate(piece.square.0+direction*2, rank));
                    }
                    break;

                }
            }
        }
        if piece.kind == PieceKind::PAWN && self.en_pessant_file.is_some() && piece.square.1 == (if piece.color == PieceColor::WHITE { 4 } else { 3 }) {
            let direction = (if piece.color == PieceColor::WHITE { 1 } else { -1 });
            for delta in [-1i8, 1] {
                if self.en_pessant_file.unwrap() == piece.square.0+delta {
                    moves.push(Coordinate(piece.square.0 + delta, piece.square.1 + direction));
                }

            }
        }

        moves
    }

    pub fn move_piece(&mut self, from: &Coordinate, to: &Coordinate) {
        let mut piece = self.pieces.get_mut(&from).unwrap().clone();
        piece.moved = true;
        piece.square = to.clone();
        let capture = self.pieces.get(to).is_some();
        self.pieces.insert(to.clone(), piece);
        self.pieces.remove(from);

        let distance = to.0 - from.0;
        if piece.kind == PieceKind::KING && distance.abs() > 1 {
            let direction = distance.signum();
            let mut position = to.0;
            while position < 8 && position >= 0 {
                position += direction;
                let coordinate = Coordinate(position, to.1);
                if self.pieces.get(&coordinate).is_none() { continue };
                self.move_piece(&coordinate, &Coordinate(from.0+direction, from.1));
                break;
            }
        }
        if piece.kind == PieceKind::PAWN && !capture && from.0 != to.0 {
            self.pieces.remove(&Coordinate(to.0, from.1));
        }

        self.en_pessant_file = None;
        let vdistance = to.1 - from.1;
        if piece.kind == PieceKind::PAWN && vdistance.abs() > 1 {
            self.en_pessant_file = Some(piece.square.0);
        }
    }

    pub fn flip_on_move(&mut self) {
        self.turn_number += 1;
        self.on_move = match self.on_move {
            PieceColor::WHITE => PieceColor::BLACK,
            PieceColor::BLACK => PieceColor::WHITE
        }
    }
}