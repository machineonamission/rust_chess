use colored::*;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Color {
    Black,
    White,
}

pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Clone)]
pub struct Move {
    from: Square,
    to: Square,
    capture: Option<PieceType>,
    castle: bool,
    promotion: Option<PieceType>,
    en_passant_able: bool,
}
type RowCol = (i8,i8);

fn rowcol_to_square(to: RowCol) -> Option<Square> {
    let (row, col) = to;
    if row >= 0 && row < 8 && col >= 0 && col < 8 {
        Some(Square((row * 8 + col) as usize))
    } else {
        None
    }
}

fn square_to_rowcol(square: &Square) -> RowCol {
    ((square.0 / 8) as i8, (square.0 % 8) as i8)
}

impl Game {
    /*fn construct_move(&self, from: Square, to: Square) -> Option<Move> {
            let piece;
            match &self.board[from.0] {
                None => {
                    return None
                }
                Some(p) => {
                    piece = p;
                }
            }
            let capture = &self.board[to.0];
            let capture_type;
            match capture {
                None => {
                    capture_type = None;
                }
                Some(p) => {
                    if piece.color == p.color {
                        return None;
                    } else {
                        capture_type = Some(p.piece_type);
                    }
                }
            }
            Some(Move {
                from,
                to,
                capture: capture_type,
                castle: false,
                promotion: None,
            })
        }
        fn construct_move_from_rowcol(&self, fromrow: i8, fromcol: i8, torow: i8, tocol: i8) -> Option<Move> {
            let from = rowcol_to_square(fromrow, fromcol);
            let to = rowcol_to_square(torow, tocol);
            self.construct_move(from?,to?)
        }*/
    fn piece_at_square(&self, square: &Square) -> &Option<Piece> {
        &self.board[square.0]
    }
    fn generic_move(&self, from: &Square, to: RowCol) -> Option<Move> {
        let to = rowcol_to_square(to)?;
        // unwrap is fine here because from should always be valid
        let color = self.piece_at_square(from).unwrap().color;
        let capture = self.piece_at_square(&to);
        match capture {
            None => {
                Some(Move {
                    from: *from,
                    to,
                    capture: None,
                    castle: false,
                    promotion: None,
                    en_passant_able: false,
                })
            }
            Some(capture_piece) => {
                if capture_piece.color != color {
                    Some(Move {
                        from: *from,
                        to,
                        capture: Some(capture_piece.piece_type),
                        castle: false,
                        promotion: None,
                        en_passant_able: false,
                    })
                } else {
                    None
                }
            }
        }
    }

    fn legal_moves_on_square(&self, square: Square) -> Vec<Move> {
        let piece = self.piece_at_square(&square);
        let mut moves = vec!();
        if let Some(piece_some) = piece {
            let (row, col) = square_to_rowcol(&square);
            match piece_some.piece_type {
                PieceType::Pawn => {
                    // if to increase row or decrease row
                    let direction: i8 = match piece_some.color {
                        Color::Black => { 1 }
                        Color::White => { -1 }
                    };
                    let mut pawn_moves: Vec<Move> = vec!();
                    let torow = row + direction;
                    // diagonal captures
                    for capture_direction in [-1i8, 1i8] {
                        // if the diagonal is a valid square
                        if let Some(capture_square) = rowcol_to_square((torow, col + capture_direction)) {
                            // if there's a piece on the diagonal
                            if let Some(capture) = self.piece_at_square(&capture_square) {
                                // if the piece is captureable
                                if capture.color != piece_some.color {
                                    pawn_moves.push(Move {
                                        from: square,
                                        to: capture_square,
                                        capture: Some(capture.piece_type),
                                        castle: false,
                                        promotion: None,
                                        en_passant_able: false,
                                    });
                                }
                            }
                        }
                    }
                    // if directly ahead is empty
                    // unwrap is safe because there's no reason this would ever be invalid
                    let one_ahead = rowcol_to_square((torow, col)).unwrap();
                    if let None = self.piece_at_square(&one_ahead) {
                        pawn_moves.push(Move {
                            from: square,
                            to: one_ahead,
                            capture: None,
                            castle: false,
                            promotion: None,
                            en_passant_able: false,
                        });
                        // this can only happen if the last square was empty and at initial rows
                        if (row == 1 && piece_some.color == Color::White) || (row == 6 && piece_some.color == Color::Black) {
                            let two_ahead = rowcol_to_square((row + direction * 2, col)).unwrap();
                            if let None = self.piece_at_square(&one_ahead) {
                                pawn_moves.push(Move {
                                    from: square,
                                    to: two_ahead,
                                    capture: None,
                                    castle: false,
                                    promotion: None,
                                    en_passant_able: true,
                                });
                            }
                        }
                    }
                    // pawns cant move backwards so i dont need to validate this for color
                    let promotion = torow == 7 || torow == 0;
                    for mut mov in pawn_moves {
                        if promotion {
                            mov.promotion = Some(PieceType::Queen);
                            moves.push(mov.clone());
                            mov.promotion = Some(PieceType::Knight);
                            moves.push(mov.clone());
                        } else {
                            moves.push(mov);
                        }
                    }
                }
                PieceType::Knight => {
                    let mut knight_moves: [RowCol; 8] = [(0, 0); 8];
                    let mut i: usize = 0;
                    for big in [-2i8, 2i8] {
                        for small in [-1i8, 1i8] {
                            knight_moves[i] = (big, small);
                            i += 1;
                            knight_moves[i] = (small, big);
                            i += 1;
                        }
                    }
                    for mov in knight_moves {
                        if let Some(m) = self.generic_move(&square, mov) {
                            moves.push(m);
                        }
                    }
                }
                _ => {

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