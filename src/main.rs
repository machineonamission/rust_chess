mod game;
mod svg_to_texture;

use glam::{vec2, Vec2};

use macroquad::prelude::*;

const BACKGROUND: Color = color_u8!(0x16, 0x14, 0x12, 0xff);
const LIGHT_SQUARE: Color = color_u8!(0xf0, 0xd9, 0xb5, 0xff);
const DARK_SQUARE: Color = color_u8!(0xb5, 0x88, 0x63, 0xff);

fn window_conf() -> Conf {
    Conf {
        window_title: "Chess".to_owned(),
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main("Chess")]
async fn main() {
    clear_background(WHITE);
    draw_text("Loading...", screen_width() / 2f32, screen_height() / 2f32, 30f32, BLACK);
    next_frame().await;
    let black_bishop = svg_to_texture::texture_from_file("assets/bB.svg");
    let black_king = svg_to_texture::texture_from_file("assets/bK.svg");
    let black_knight = svg_to_texture::texture_from_file("assets/bN.svg");
    let black_pawn = svg_to_texture::texture_from_file("assets/bP.svg");
    let black_queen = svg_to_texture::texture_from_file("assets/bQ.svg");
    let black_rook = svg_to_texture::texture_from_file("assets/bR.svg");
    let white_bishop = svg_to_texture::texture_from_file("assets/wB.svg");
    let white_king = svg_to_texture::texture_from_file("assets/wK.svg");
    let white_knight = svg_to_texture::texture_from_file("assets/wN.svg");
    let white_pawn = svg_to_texture::texture_from_file("assets/wP.svg");
    let white_queen = svg_to_texture::texture_from_file("assets/wQ.svg");
    let white_rook = svg_to_texture::texture_from_file("assets/wR.svg");
    let mut game = game::default_game();
    let mut moving_piece: Option<usize> = None;
    loop {
        clear_background(BACKGROUND);
        let width = screen_width();
        let height = screen_height();
        let board_size = f32::min(width, height);
        let top_left = ((width - board_size) / 2f32, (height - board_size) / 2f32);
        let square_size = board_size / 8f32;
        for col in 0..8 {
            for row in 0..8 {
                draw_rectangle(top_left.0 + row as f32 * square_size,
                               top_left.1 + col as f32 * square_size,
                               square_size,
                               square_size,
                               if row % 2 == col % 2 {
                                   LIGHT_SQUARE
                               } else {
                                   DARK_SQUARE
                               });
                // skip drawing moving piece
                if let Some(m) = moving_piece {
                    if m == row + col * 8 {
                        continue
                    }
                }
                if let Some(p) = &game.board[row + col * 8] {
                    draw_texture_ex(
                        match p.color {
                            game::Color::Black => {
                                match p.piece_type {
                                    game::PieceType::Pawn => { &black_pawn }
                                    game::PieceType::Knight => { &black_knight }
                                    game::PieceType::Bishop => { &black_bishop }
                                    game::PieceType::Rook => { &black_rook }
                                    game::PieceType::Queen => { &black_queen }
                                    game::PieceType::King => { &black_king }
                                }
                            }
                            game::Color::White => {
                                match p.piece_type {
                                    game::PieceType::Pawn => { &white_pawn }
                                    game::PieceType::Knight => { &white_knight }
                                    game::PieceType::Bishop => { &white_bishop }
                                    game::PieceType::Rook => { &white_rook }
                                    game::PieceType::Queen => { &white_queen }
                                    game::PieceType::King => { &white_king }
                                }
                            }
                        },
                        top_left.0 + row as f32 * square_size,
                        top_left.1 + col as f32 * square_size,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(square_size, square_size)),
                            ..Default::default()
                        },
                    );
                };
            }
        }
        next_frame().await;
    }
}
