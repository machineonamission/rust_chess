mod game;
mod svg_to_texture;

use glam::{vec2};

use macroquad::prelude::*;
use crate::game::Color::White;

const BACKGROUND: Color = color_u8!(0x16, 0x14, 0x12, 0xff);
const LIGHT_SQUARE: Color = color_u8!(0xf0, 0xd9, 0xb5, 0xff);
const DARK_SQUARE: Color = color_u8!(0xb5, 0x88, 0x63, 0xff);

const SELECTED: Color = color_u8!(20, 85, 30, 0x7f);

fn window_conf() -> Conf {
    Conf {
        window_title: "Chess".to_owned(),
        // high_dpi: true,
        ..Default::default()
    }
}


#[macroquad::main(window_conf)]
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

    let mut moving_piece: Option<game::Square> = None;
    let mut selected_piece: Option<game::Square> = None;
    let mut selected_targets: Vec<game::Square> = vec!();

    let draw_piece = |p: &game::Piece, x: f32, y: f32, size: f32, color: Color| {
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
                White => {
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
            x,
            y,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(size, size)),
                ..Default::default()
            },
        );
    };
    loop {
        clear_background(BACKGROUND);
        let width = screen_width();
        let height = screen_height();
        let board_size = f32::min(width, height);
        let top_left = ((width - board_size) / 2f32, (height - board_size) / 2f32);
        let square_size = board_size / 8f32;

        let mouse_pos = mouse_position();
        let row = ((mouse_pos.1 - top_left.1) / square_size).floor() as i8;
        let col = ((mouse_pos.0 - top_left.0) / square_size).floor() as i8;
        let mouse_square_option = game::is_valid_square(row, col);


        if let Some(mouse_square) = mouse_square_option {
            if is_mouse_button_pressed(MouseButton::Left) {
                if let Some(s) = selected_piece {
                    if s != mouse_square {
                        game.move_piece(&s, &mouse_square);
                    }
                    moving_piece = None;
                    selected_piece = None;
                    selected_targets = vec!();
                } else if let Some(_) = game.piece_at_square(&mouse_square) {
                    moving_piece = Some(mouse_square);
                    selected_piece = Some(mouse_square);
                    for mov in game.legal_moves_on_square(mouse_square) {
                        selected_targets.push(mov.to)
                    }
                    selected_targets.dedup();
                } else {
                    moving_piece = None;
                    selected_piece = None;
                    selected_targets = vec!();
                }
            }
            if is_mouse_button_released(MouseButton::Left) {
                if let Some(p) = moving_piece {
                    if p == mouse_square {
                        moving_piece = None;
                        // intentionally don't touch selected piece
                    } else if let Some(s) = selected_piece {
                        game.move_piece(&s, &mouse_square);
                        moving_piece = None;
                        selected_piece = None;
                        selected_targets = vec!();
                    }
                }
            }
        } else if is_mouse_button_released(MouseButton::Left) {
            moving_piece = None;
            selected_piece = None;
            selected_targets = vec!();
        }

        for row in 0..8 {
            for col in 0..8 {
                let mut selected = false;
                if let Some(m) = selected_piece {
                    if m == (row,col) {
                        selected = true;
                    }
                }
                draw_rectangle(top_left.0 + col as f32 * square_size,
                               top_left.1 + row as f32 * square_size,
                               square_size,
                               square_size,
                               if row % 2 == col % 2 {
                                   LIGHT_SQUARE
                               } else {
                                   DARK_SQUARE
                               });
                if selected {
                    draw_rectangle(top_left.0 + col as f32 * square_size,
                                   top_left.1 + row as f32 * square_size,
                                   square_size,
                                   square_size,
                                   SELECTED);
                }
                if let Some(p) = game.piece_at_square(&(row,col)) {
                    // draw moving piece at half opacity
                    let mut color = WHITE;
                    if let Some(m) = moving_piece {
                        if m.0 == row + col * 8 {
                            color = color_u8!(0xff, 0xff, 0xff, 0x7f);
                        }
                    }
                    draw_piece(
                        p,
                        top_left.0 + col as f32 * square_size,
                        top_left.1 + row as f32 * square_size,
                        square_size,
                        color,
                    );
                };
            }
        }
        // draw selected squares
        for (row,col) in &selected_targets {
            let offset = (top_left.0 + *col as f32 * square_size, top_left.1 + *row as f32 * square_size);
            if game.piece_at_square(&(*row,*col) /* this is horrible */).is_some() { // a piece that can be captured
                draw_triangle(
                    vec2(offset.0, offset.1),
                    vec2(offset.0 + square_size / 4f32, offset.1),
                    vec2(offset.0, offset.1 + square_size / 4f32),
                    SELECTED
                );
                draw_triangle(
                    vec2(offset.0 + square_size, offset.1),
                    vec2(offset.0 + square_size - square_size / 4f32, offset.1),
                    vec2(offset.0 + square_size, offset.1 + square_size / 4f32),
                    SELECTED
                );
                draw_triangle(
                    vec2(offset.0, offset.1 + square_size),
                    vec2(offset.0 + square_size / 4f32, offset.1 + square_size),
                    vec2(offset.0, offset.1  + square_size - square_size / 4f32),
                    SELECTED
                );
                draw_triangle(
                    vec2(offset.0 + square_size, offset.1 + square_size),
                    vec2(offset.0 + square_size - square_size / 4f32, offset.1 + square_size),
                    vec2(offset.0 + square_size, offset.1 + square_size - square_size / 4f32),
                    SELECTED
                );
            } else { // empty square
                draw_circle(offset.0 + square_size / 2f32,
                            offset.1 + square_size / 2f32,
                            square_size / 10f32, SELECTED)
            }

        }

        // draw held piece
        if let Some(p) = moving_piece {
            if let Some(m) = game.piece_at_square(&p) {
                draw_piece(m, mouse_pos.0 - square_size / 2f32, mouse_pos.1 - square_size / 2f32, square_size, WHITE)
            }
        }
        next_frame().await;
    }
}