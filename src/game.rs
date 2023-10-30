use colored::*;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

pub enum Color {
    Black,
    White,
}

pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

#[derive(Debug)]
pub struct Square(usize);

impl Square {
    pub fn new(value: usize) -> Self {
        assert!(value <= 63);
        Square(value)
    }
    fn row(self) -> usize {
        self.0 % 8
    }
    fn col(self) -> usize {
        self.0 / 8usize // self.0 and 8 are ints so this should floor divide
    }
}


pub struct CastlingRights {
    pub white_queenside: bool,
    pub white_kingside: bool,
    pub black_queenside: bool,
    pub black_kingside: bool,
}

pub struct Game {
    pub board: [Option<Piece>; 8 * 8],
    pub turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_target_square: Option<Square>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
    pub moves: Vec<Move>,
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (index, piece) in self.board.iter().enumerate() {
            if index > 0 && index % 8 == 0 {
                f.write_str("\n")?;
            }
            let piecestring = match piece {
                Some(p) => {
                    match p.color {
                        Color::Black => {
                            match p.piece_type {
                                PieceType::Pawn => { "p" }
                                PieceType::Knight => { "n" }
                                PieceType::Bishop => { "b" }
                                PieceType::Rook => { "r" }
                                PieceType::Queen => { "q" }
                                PieceType::King => { "k" }
                            }
                        }
                        Color::White => {
                            match p.piece_type {
                                PieceType::Pawn => { "P" }
                                PieceType::Knight => { "N" }
                                PieceType::Bishop => { "B" }
                                PieceType::Rook => { "R" }
                                PieceType::Queen => { "Q" }
                                PieceType::King => { "K" }
                            }
                        }
                    }
                }
                None => {
                    " "
                }
            };
            let colored = if index % 2 == (index / 8) % 2 {
                piecestring.on_truecolor(0xb5, 0x88, 0x63).black()
            } else {
                piecestring.on_truecolor(0xf0, 0xd9, 0xb5).black()
            };
            f.write_fmt(
                format_args!("{}", colored)
            )?;
        }
        Ok(())
    }
}

pub struct Move {
    from: Square,
    to: Square,
    capture: Option<Piece>,
    castle: bool,
    promotion: Option<Piece>,
}

fn rowcol_to_square(row: usize, col: usize) -> Square {
    debug_assert!(row < 8);
    debug_assert!(col < 8);
    Square(row * 8 + col)
}

fn square_to_rowcol(square: Square) -> (usize, usize) {
    (square.0 / 8, square.0 % 8)
}

impl Game {
    fn construct_move(&self, from: Square, to: Square) -> Option<Move> {
        
    }
    fn legal_moves_on_square(&self, square: Square) -> Vec<Move> {
        let piece = &self.board[square.0];
        let moves = vec!();
        if let Some(p) = piece {
            let (row, col) = square_to_rowcol(square);
            match p.piece_type {
                PieceType::Pawn => {
                    // if to increase row or decrease row
                    let direction = match p.color {
                        Color::Black => { 1 }
                        Color::White => { -1 }
                    };
                }
            }
        }
        moves
    }
}

const INITIAL_ROW: [PieceType; 8] = [
    PieceType::Rook,
    PieceType::Knight,
    PieceType::Bishop,
    PieceType::Queen,
    PieceType::King,
    PieceType::Bishop,
    PieceType::Knight,
    PieceType::Rook
];

pub fn default_game() -> Game {
    const INIT: Option<Piece> = None;
    let mut game = Game {
        board: [INIT; 64],
        turn: Color::Black,
        castling_rights: CastlingRights {
            white_queenside: true,
            white_kingside: true,
            black_queenside: true,
            black_kingside: true,
        },
        en_passant_target_square: None,
        halfmove_clock: 0,
        fullmove_number: 0,
        moves: vec!(),
    };
    // initialize top and bottom rows with the starting arrangement
    for (index, piecetype) in INITIAL_ROW.iter().enumerate() {
        game.board[index] = Some(Piece {
            piece_type: piecetype.clone(),
            color: Color::Black,
        });
        game.board[index + 56] = Some(Piece {
            piece_type: piecetype.clone(),
            color: Color::White,
        });
    }
    // initialize pawns
    for i in 0..8 {
        game.board[i + 8] = Some(Piece {
            piece_type: PieceType::Pawn,
            color: Color::Black,
        });
        game.board[i + 48] = Some(Piece {
            piece_type: PieceType::Pawn,
            color: Color::White,
        });
    }
    game
}