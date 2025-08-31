use crate::game::{GoGame};
use crate::sgf_parser::Player;
use crossterm::event::{KeyCode, KeyEvent};
use crate::sgf_parser::sgf_to_string;
use crate::ui::{UiMode};

pub enum UiAction {
    Continue,
    ChangeMode(UiMode),
    Quit,
}

pub fn handle_normal_input(key: &KeyEvent, game: &mut GoGame) -> UiAction {
    match key.code {
        KeyCode::Char('i') => {
            let color = if game.move_idx > 0 && game.move_idx <= game.moves.len() {
                match game.moves[game.move_idx - 1].player {
                    Player::Black => Player::White,
                    Player::White => Player::Black,
                }
            } else {
                Player::Black
            };
            UiAction::ChangeMode(UiMode::InsertMoveInput { input: String::new(), color })
        },
        KeyCode::Char('x') => {
            if game.move_idx > 0 && game.move_idx <= game.moves.len() {
                let idx = game.move_idx - 1;
                game.moves.remove(idx);
                game.original_sgf.moves.remove(idx);
                if game.move_idx > 1 {
                    game.move_idx -= 1;
                }
                game.apply_moves(game.move_idx);
                let _ = game.save_to_file();
            }
            UiAction::Continue
        },
        KeyCode::Char('q') => UiAction::Quit,
        KeyCode::Char('n') | KeyCode::Right => {
            game.next_move();
            UiAction::Continue
        },
        KeyCode::Char('p') | KeyCode::Left => {
            game.prev_move();
            UiAction::Continue
        },
        KeyCode::Char(']') => {
            if !game.moves.is_empty() && game.move_idx < game.moves.len() {
                let mut idx = game.move_idx;
                while idx < game.moves.len() {
                    if let Some(c) = &game.moves[idx].comment {
                        if !c.trim().is_empty() {
                            game.move_idx = idx + 1;
                            game.apply_moves(game.move_idx);
                            break;
                        }
                    }
                    idx += 1;
                }
            }
            UiAction::Continue
        },
        KeyCode::Char('[') => {
            if !game.moves.is_empty() && game.move_idx > 1 {
                let mut idx = game.move_idx - 2;
                loop {
                    if let Some(c) = &game.moves[idx].comment {
                        if !c.trim().is_empty() {
                            game.move_idx = idx + 1;
                            game.apply_moves(game.move_idx);
                            break;
                        }
                    }
                    if idx == 0 { break; }
                    idx -= 1;
                }
            }
            UiAction::Continue
        },
        KeyCode::Char('g') => UiAction::ChangeMode(UiMode::GotoMoveInput { input: String::new() }),
        KeyCode::Char('m') => UiAction::ChangeMode(UiMode::ModifyMoveInput { input: String::new() }),
        KeyCode::Char('h') => UiAction::ChangeMode(UiMode::HotkeyHelp),
        KeyCode::Char('/') => UiAction::ChangeMode(UiMode::SearchCoordInput { input: String::new() }),
        KeyCode::Char('c') => {
            if game.move_idx > 0 && game.move_idx <= game.moves.len() {
                let comment = game.moves[game.move_idx - 1].comment.clone().unwrap_or_default();
                // Extract only the comment part (before first underscore)
                let comment_only = comment.split('_').next().unwrap_or("").to_string();
                UiAction::ChangeMode(UiMode::EditCommentInput { input: comment_only })
            } else {
                UiAction::Continue
            }
        },
        KeyCode::Char('l') => {
            if game.move_idx > 0 && game.move_idx <= game.moves.len() {
                let comment = game.moves[game.move_idx - 1].comment.clone().unwrap_or_default();
                // Extract labels (everything after first underscore)
                let labels = if let Some(underscore_pos) = comment.find('_') {
                    comment[underscore_pos + 1..].replace('_', ",")
                } else {
                    String::new()
                };
                UiAction::ChangeMode(UiMode::EditLabelInput { input: labels })
            } else {
                UiAction::Continue
            }
        },
        KeyCode::Char('t') => {
            let input = game.current_triangles()
                .iter()
                .map(|(x, y)| {
                    let c = |v| (b'a' + v as u8) as char;
                    format!("{}{}", c(*y), c(*x))
                })
                .collect::<Vec<_>>()
                .join(",");
            UiAction::ChangeMode(UiMode::EditTrianglesInput { input })
        },
        _ => UiAction::Continue
    }
}

// GotoMoveInput handler
pub fn handle_goto_move_input(key: &KeyEvent, input: &mut String, game: &mut GoGame) -> Option<UiMode> {
    match key.code {
        KeyCode::Esc => Some(UiMode::Normal),
        KeyCode::Enter => {
            if let Ok(num) = input.parse::<usize>() {
                if num <= game.moves.len() {
                    game.move_idx = num;
                    game.apply_moves(game.move_idx);
                }
            }
            Some(UiMode::Normal)
        },
        KeyCode::Char(c) if c.is_ascii_digit() => {
            input.push(c);
            None
        },
        KeyCode::Backspace => {
            input.pop();
            None
        },
        _ => None
    }
}

// ModifyMoveInput handler
pub fn handle_modify_move_input(key: &KeyEvent, input: &mut String, game: &mut GoGame) -> Option<UiMode> {
    match key.code {
        KeyCode::Esc => Some(UiMode::Normal),
        KeyCode::Enter => {
            if input.len() == 2 {
                let y = (input.chars().nth(0).unwrap() as u8).wrapping_sub(b'a') as usize;
                let x = (input.chars().nth(1).unwrap() as u8).wrapping_sub(b'a') as usize;
                if x < game.board_size && y < game.board_size && game.move_idx > 0 && game.move_idx <= game.moves.len() {
                    let idx = game.move_idx - 1;
                    game.moves[idx].x = x;
                    game.moves[idx].y = y;
                    game.original_sgf.moves[idx].x = x;
                    game.original_sgf.moves[idx].y = y;
                    game.apply_moves(game.move_idx);
                    let _ = game.save_to_file();
                }
            }
            Some(UiMode::Normal)
        },
        KeyCode::Char(c) if c.is_ascii_lowercase() && input.len() < 2 => {
            input.push(c);
            None
        },
        KeyCode::Backspace => {
            input.pop();
            None
        },
        _ => None
    }
}

// SearchCoordInput handler
pub fn handle_search_coord_input(key: &KeyEvent, input: &mut String, game: &mut GoGame) -> Option<UiMode> {
    match key.code {
        KeyCode::Esc => Some(UiMode::Normal),
        KeyCode::Enter => {
            if input.len() == 2 {
                let y = (input.chars().nth(0).unwrap() as u8).wrapping_sub(b'a') as usize;
                let x = (input.chars().nth(1).unwrap() as u8).wrapping_sub(b'a') as usize;
                if x < game.board_size && y < game.board_size {
                    if let Some(idx) = game.moves.iter().position(|mv| mv.x == x && mv.y == y) {
                        game.move_idx = idx + 1;
                        game.apply_moves(game.move_idx);
                    }
                }
            }
            Some(UiMode::Normal)
        },
        KeyCode::Char(c) if c.is_ascii_lowercase() && input.len() < 2 => {
            input.push(c);
            None
        },
        KeyCode::Backspace => {
            input.pop();
            None
        },
        _ => None
    }
}

// EditCommentInput handler
pub fn handle_edit_comment_input(key: &KeyEvent, input: &mut String, game: &mut GoGame) -> Option<UiMode> {
    match key.code {
        KeyCode::Esc => Some(UiMode::Normal),
        KeyCode::Enter => {
            if game.move_idx > 0 && game.move_idx <= game.moves.len() {
                let idx = game.move_idx - 1;
                let existing_comment = game.moves[idx].comment.clone().unwrap_or_default();
                // Preserve existing labels (everything after first underscore)
                let existing_labels = if let Some(underscore_pos) = existing_comment.find('_') {
                    existing_comment[underscore_pos..].to_string()
                } else {
                    String::new()
                };
                
                let new_comment = if input.trim().is_empty() {
                    if existing_labels.is_empty() { None } else { Some(existing_labels) }
                } else {
                    Some(format!("{}{}", input.trim(), existing_labels))
                };
                
                game.moves[idx].comment = new_comment.clone();
                game.original_sgf.moves[idx].comment = new_comment;
                let _ = game.save_to_file();
            }
            Some(UiMode::Normal)
        },
        KeyCode::Backspace => {
            input.pop();
            None
        },
        KeyCode::Char(c) => {
            // Block underscores in comment input
            if c != '_' {
                input.push(c);
            }
            None
        },
        _ => None
    }
}

// InsertMoveInput handler
pub fn handle_insert_move_input(key: &KeyEvent, input: &mut String, color: &mut Player, game: &mut GoGame) -> Option<UiMode> {
    match key.code {
        KeyCode::Esc => Some(UiMode::Normal),
        KeyCode::Tab => {
            *color = match color {
                Player::Black => Player::White,
                Player::White => Player::Black,
            };
            None
        },
        KeyCode::Enter => {
            if input.len() == 2 {
                let y = (input.chars().nth(0).unwrap() as u8).wrapping_sub(b'a') as usize;
                let x = (input.chars().nth(1).unwrap() as u8).wrapping_sub(b'a') as usize;
                if x < game.board_size && y < game.board_size {
                    let new_move = crate::sgf_parser::Move {
                        player: color.clone(),
                        x,
                        y,
                        comment: None,
                        triangles: vec![],
                    };
                    game.moves.insert(game.move_idx, new_move.clone());
                    game.original_sgf.moves.insert(game.move_idx, new_move);
                    game.move_idx += 1;
                    game.apply_moves(game.move_idx);
                    let _ = game.save_to_file();
                }
            }
            Some(UiMode::Normal)
        },
        KeyCode::Char(c) if c.is_ascii_lowercase() && input.len() < 2 => {
            input.push(c);
            None
        },
        KeyCode::Backspace => {
            input.pop();
            None
        },
        _ => None
    }
}

// EditLabelInput handler
pub fn handle_edit_label_input(key: &KeyEvent, input: &mut String, game: &mut GoGame) -> Option<UiMode> {
    match key.code {
        KeyCode::Esc => Some(UiMode::Normal),
        KeyCode::Enter => {
            if game.move_idx > 0 && game.move_idx <= game.moves.len() {
                let idx = game.move_idx - 1;
                let existing_comment = game.moves[idx].comment.clone().unwrap_or_default();
                // Extract existing comment part (before first underscore)
                let existing_comment_part = existing_comment.split('_').next().unwrap_or("").to_string();
                
                let new_comment = if input.trim().is_empty() {
                    if existing_comment_part.is_empty() { None } else { Some(existing_comment_part) }
                } else {
                    // Convert comma-separated labels to underscore-separated
                    let labels = input.split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join("_");
                    
                    if existing_comment_part.is_empty() {
                        Some(format!("_{}", labels))
                    } else {
                        Some(format!("{}_{}", existing_comment_part, labels))
                    }
                };
                
                game.moves[idx].comment = new_comment.clone();
                game.original_sgf.moves[idx].comment = new_comment;
                let _ = game.save_to_file();
            }
            Some(UiMode::Normal)
        },
        KeyCode::Backspace => {
            input.pop();
            None
        },
        KeyCode::Char(c) => {
            input.push(c);
            None
        },
        _ => None
    }
}

pub fn handle_edit_triangles_input(key: &KeyEvent, input: &mut String, game: &mut GoGame) -> Option<UiMode> {
    match key.code {
        KeyCode::Esc => return Some(UiMode::Normal),
        KeyCode::Enter => {
            // Parse comma-separated coords
            let coords = input.split(',').filter_map(|s| {
                let s = s.trim();
                if s.len() == 2 {
                    let y = (s.chars().nth(0).unwrap() as u8).wrapping_sub(b'a') as usize;
                    let x = (s.chars().nth(1).unwrap() as u8).wrapping_sub(b'a') as usize;
                    Some((x, y))
                } else {
                    None
                }
            }).collect::<Vec<_>>();
            if let Some(tris) = game.current_triangles_mut() {
                tris.clear();
                tris.extend(&coords);
            }
            if let Some(tris) = game.original_sgf.moves.get_mut(game.move_idx.saturating_sub(1)) {
                tris.triangles.clear();
                tris.triangles.extend(&coords);
            }
            if let Ok(sgf_str) = sgf_to_string(&game.original_sgf) {
                if let Some(path) = game.original_sgf_path.as_deref() {
                    let _ = std::fs::write(path, sgf_str);
                }
            }
            return Some(UiMode::Normal);
        },
        KeyCode::Char(c) => {
            input.push(c);
        },
        KeyCode::Backspace => {
            input.pop();
        },
        _ => {}
    }
    None
}
