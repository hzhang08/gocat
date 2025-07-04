use crate::game::{GoGame, Stone};
use crate::sgf_parser::sgf_to_string;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::io::{self, Stdout};

enum UiMode {
    Normal,
    GotoMoveInput { input: String },
    HotkeyHelp,
    ModifyMoveInput { input: String },
    SearchCoordInput { input: String },
    EditCommentInput { input: String },
    EditTrianglesInput { input: String },


}

pub fn run_ui(game: &mut GoGame) -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let mut mode = UiMode::Normal;
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let board = render_board(game);
            let meta = render_metadata(game);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length((game.board_size + 3) as u16),
                    Constraint::Min(3),
                ])
                .split(size);
            f.render_widget(board, chunks[0]);
            f.render_widget(meta, chunks[1]);

            // Use a reference to the current mode so popup input is live
            match &mode {
                UiMode::GotoMoveInput { input } => {
                    let area = centered_rect(30, 10, size);
                    let block = Block::default().title("Goto Move").borders(Borders::ALL);
                    let text = Paragraph::new(format!("Enter move #: {}", input)).block(block);
                    f.render_widget(text, area);
                }
                UiMode::HotkeyHelp => {
                    let area = centered_rect(50, 40, size);
                    let block = Block::default().title("Hotkey Help").borders(Borders::ALL);
                    let help = [
                        "q         Quit",
                        "n / →     Next move",
                        "p / ←     Previous move",
                        "g         Goto move number",
                        "m         Modify current move",
                        "/         Search for coordinate",
                        "c         Add/Edit move comment",
                        "t         Add/Edit triangles",
                        "h         Show this help",
                        "Esc/Enter Close this help",
                    ].join("\n");
                    let text = Paragraph::new(help).block(block);
                    f.render_widget(text, area);
                }
                UiMode::ModifyMoveInput { input } => {
                    let area = centered_rect(30, 10, size);
                    let block = Block::default().title("Modify Move").borders(Borders::ALL);
                    let text = Paragraph::new(format!("Enter coords (e.g., dd): {}", input)).block(block);
                    f.render_widget(text, area);
                }
                UiMode::EditCommentInput { input } => {
                    let area = centered_rect(50, 10, size);
                    let block = Block::default().title("Edit Comment").borders(Borders::ALL);
                    let text = Paragraph::new(format!("Edit comment: {}", input)).block(block);
                    f.render_widget(text, area);
                }
                UiMode::EditTrianglesInput { input } => {
                    let area = centered_rect(60, 10, size);
                    let block = Block::default().title("Edit Triangles").borders(Borders::ALL);
                    let text = Paragraph::new(format!("Comma-separated coords (e.g., dd,ee,fg): {}", input)).block(block);
                    f.render_widget(text, area);
                }
                UiMode::SearchCoordInput { input } => {
                    let area = centered_rect(30, 10, size);
                    let block = Block::default().title("Search Coord").borders(Borders::ALL);
                    let text = Paragraph::new(format!("Enter coords (e.g., dd): {}", input)).block(block);
                    f.render_widget(text, area);
                }
                _ => {}
            }
        })?;
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let mode_ref = &mut mode;
                    match mode_ref {
                        UiMode::Normal => match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('n') | KeyCode::Right => game.next_move(),
                            KeyCode::Char('p') | KeyCode::Left => game.prev_move(),
                            KeyCode::Char('g') => *mode_ref = UiMode::GotoMoveInput { input: String::new() },
                            KeyCode::Char('m') => *mode_ref = UiMode::ModifyMoveInput { input: String::new() },
                            KeyCode::Char('h') => *mode_ref = UiMode::HotkeyHelp,
                            KeyCode::Char('/') => *mode_ref = UiMode::SearchCoordInput { input: String::new() },
                            KeyCode::Char('c') => {
                                if game.move_idx > 0 && game.move_idx <= game.moves.len() {
                                    let comment = game.moves[game.move_idx - 1].comment.clone().unwrap_or_default();
                                    *mode_ref = UiMode::EditCommentInput { input: comment };
                                }
                            },
                            KeyCode::Char('t') => {
                                // Prepopulate with current move's triangles
                                let input = game.current_triangles()
                                    .iter()
                                    .map(|(x, y)| {
                                        let c = |v| (b'a' + v as u8) as char;
                                        format!("{}{}", c(*y), c(*x))
                                    })
                                    .collect::<Vec<_>>()
                                    .join(",");
                                *mode_ref = UiMode::EditTrianglesInput { input };
                            },
                            _ => {}
                        },
                        UiMode::GotoMoveInput { input } => match key.code {
                            KeyCode::Esc => *mode_ref = UiMode::Normal,
                            KeyCode::Enter => {
                                if let Ok(num) = input.parse::<usize>() {
                                    if num <= game.moves.len() {
                                        game.move_idx = num;
                                        game.apply_moves(game.move_idx);
                                    }
                                }
                                *mode_ref = UiMode::Normal;
                            },
                            KeyCode::Char(c) if c.is_ascii_digit() => {
                                input.push(c);
                            },
                            KeyCode::Backspace => {
                                input.pop();
                            },
                            _ => {}
                        },
                        UiMode::HotkeyHelp => match key.code {
                            KeyCode::Esc | KeyCode::Enter => *mode_ref = UiMode::Normal,
                            _ => {}
                        },
                        UiMode::SearchCoordInput { input } => match key.code {
                            KeyCode::Esc => *mode_ref = UiMode::Normal,
                            KeyCode::Enter => {
                                if input.len() == 2 {
                                    let y = (input.chars().nth(0).unwrap() as u8).wrapping_sub(b'a') as usize;
                                    let x = (input.chars().nth(1).unwrap() as u8).wrapping_sub(b'a') as usize;
                                    if x < game.board_size && y < game.board_size {
                                        // Search for move with (x, y)
                                        if let Some(idx) = game.moves.iter().position(|mv| mv.x == x && mv.y == y) {
                                            game.move_idx = idx + 1;
                                            game.apply_moves(game.move_idx);
                                        }
                                    }
                                }
                                *mode_ref = UiMode::Normal;
                            },
                            KeyCode::Char(c) if c.is_ascii_lowercase() && input.len() < 2 => {
                                input.push(c);
                            },
                            KeyCode::Backspace => {
                                input.pop();
                            },
                            _ => {}
                        },
                        UiMode::EditTrianglesInput { input } => match key.code {
                            KeyCode::Esc => *mode_ref = UiMode::Normal,
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
                                *mode_ref = UiMode::Normal;
                            },
                            KeyCode::Char(c) => {
                                input.push(c);
                            },
                            KeyCode::Backspace => {
                                input.pop();
                            },
                            _ => {}
                        },
                        UiMode::EditCommentInput { input } => match key.code {
                            KeyCode::Esc => *mode_ref = UiMode::Normal,
                            KeyCode::Enter => {
                                if game.move_idx > 0 && game.move_idx <= game.moves.len() {
                                    let idx = game.move_idx - 1;
                                    let trimmed = if input.trim().is_empty() { None } else { Some(input.clone()) };
                                    game.moves[idx].comment = trimmed.clone();
                                    game.original_sgf.moves[idx].comment = trimmed;
                                    if let Ok(sgf_str) = sgf_to_string(&game.original_sgf) {
                                        let _ = std::fs::write("[blockchain]vs[zorba3256]1745041370030031153.sgf", sgf_str);
                                    }
                                }
                                *mode_ref = UiMode::Normal;
                            },
                            KeyCode::Char(c) => {
                                input.push(c);
                            },
                            KeyCode::Backspace => {
                                input.pop();
                            },
                            _ => {}
                        },
                        UiMode::ModifyMoveInput { input } => match key.code {
                            KeyCode::Esc => *mode_ref = UiMode::Normal,
                            KeyCode::Enter => {
                                if input.len() == 2 && game.move_idx > 0 && game.move_idx <= game.moves.len() {
                                    let y = (input.chars().nth(0).unwrap() as u8).wrapping_sub(b'a') as usize;
                                    let x = (input.chars().nth(1).unwrap() as u8).wrapping_sub(b'a') as usize;
                                    if x < game.board_size && y < game.board_size {
                                        let idx = game.move_idx - 1;
                                        game.moves[idx].x = x;
                                        game.moves[idx].y = y;
                                        game.original_sgf.moves[idx].x = x;
                                        game.original_sgf.moves[idx].y = y;
                                        game.apply_moves(game.move_idx);
                                        // Save SGF
                                        if let Ok(sgf_str) = sgf_to_string(&game.original_sgf) {
                                            let _ = std::fs::write("[blockchain]vs[zorba3256]1745041370030031153.sgf", sgf_str);
                                        }
                                    }
                                }
                                *mode_ref = UiMode::Normal;
                            },
                            KeyCode::Char(c) if c.is_ascii_lowercase() && input.len() < 2 => {
                                input.push(c);
                            },
                            KeyCode::Backspace => {
                                input.pop();
                            },
                            _ => {}
                        },
                    }
                }
            }
        }
    }
    restore_terminal(&mut terminal)
}

// Helper to center a popup
fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    use ratatui::layout::{Rect};
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    let vertical = popup_layout[1];
    let horizontal_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical);
    horizontal_layout[1]
}

fn render_board(game: &GoGame) -> Paragraph<'_> {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Span, Line, Text};
    let size = game.board_size;
    let mut lines: Vec<Line> = Vec::with_capacity(size + 1);
    // Determine current move coordinates if available
    let (cur_x, cur_y) = if game.move_idx > 0 && game.move_idx <= game.moves.len() {
        let mv = &game.moves[game.move_idx - 1];
        (mv.x, mv.y)
    } else {
        (usize::MAX, usize::MAX)
    };
    // Top coordinate row
    let mut top_spans = Vec::with_capacity(size * 2 + 2);
    // Shift left so letters align with lines
    top_spans.push(Span::raw(" "));
    for x in 0..size {
        top_spans.push(Span::raw(" "));
        let letter = ((b'a' + x as u8) as char).to_string();
        top_spans.push(Span::styled(letter, Style::default().fg(Color::Yellow)));
    }
    lines.push(Line::from(top_spans));
    // Board rows with left coordinate
    for y in 0..size {
        let mut spans = Vec::with_capacity(size * 2 + 2);
        // Row letter
        let letter = ((b'a' + y as u8) as char).to_string();
        spans.push(Span::styled(format!("{} ", letter), Style::default().fg(Color::Yellow)));
        for x in 0..size {
            let triangle_here = game.current_triangles().iter().any(|&(tx, ty)| tx == x && ty == y);
            let (ch, is_grid, triangle_on_empty) = match game.board[y][x] {
                Stone::Black => {
                    if triangle_here {
                        ('▲', false, false)
                    } else {
                        ('●', false, false)
                    }
                },
                Stone::White => {
                    if triangle_here {
                        ('△', false, false)
                    } else {
                        ('○', false, false)
                    }
                },
                Stone::Empty => {
                    if triangle_here {
                        ('△', false, true)
                    } else {
                        let grid_ch = match (y, x) {
                            (0, 0) => '┌',
                            (0, xx) if xx == size - 1 => '┐',
                            (yy, 0) if yy == size - 1 => '└',
                            (yy, xx) if yy == size - 1 && xx == size - 1 => '┘',
                            (0, _) => '┬',
                            (_, 0) => '├',
                            (yy, _) if yy == size - 1 => '┴',
                            (_, xx) if xx == size - 1 => '┤',
                            _ => '┼',
                        };
                        (grid_ch, true, false)
                    }
                }
            };
            if is_grid {
                spans.push(Span::styled(ch.to_string(), Style::default().fg(Color::Blue)));
            } else if triangle_here {
                spans.push(Span::styled(ch.to_string(), Style::default().fg(Color::Yellow)));
            } else if x == cur_x && y == cur_y {
                // Highlight the current move in red
                spans.push(Span::styled(ch.to_string(), Style::default().fg(Color::Red)));
            } else {
                spans.push(Span::raw(ch.to_string()));
            }
            if x < size - 1 {
                // Horizontal line
                spans.push(Span::styled("─", Style::default().fg(Color::Blue)));
            }
        }
        lines.push(Line::from(spans));
    }
    let text = Text::from(lines);
    Paragraph::new(text)
        .block(Block::default().title("Go Board").borders(Borders::ALL))
}



fn render_metadata(game: &GoGame) -> Paragraph<'_> {
    let move_num = game.move_idx;
    let player = match game.current_player() {
        crate::sgf_parser::Player::Black => "Black",
        crate::sgf_parser::Player::White => "White",
    };
    let total_moves = game.moves.len();
    // Show coordinates of the current move if available, on the first line
    let mut coord_str = String::new();
    let mut comment_str = String::from("Comment: N/A\n");
    if move_num > 0 && move_num <= game.moves.len() {
        let mv = &game.moves[move_num - 1];
        let coord = format!("{}{}", (b'a' + mv.y as u8) as char, (b'a' + mv.x as u8) as char);
        coord_str = format!(" [{}]", coord);
        if let Some(comment) = &mv.comment {
            if !comment.trim().is_empty() {
                comment_str = format!("Comment: {}\n", comment);
            }
        }
    }
    let mut meta_str = format!("Move: {}{} / {} | Current Player: {}\n{}", move_num, coord_str, total_moves, player, comment_str);

    for (k, v) in &game.metadata {
        if k != "FF" && k != "AP" && k != "GM" && k != "HA" && k != "KM" && k != "RL" && k != "RN" && k != "TC" && k != "TM" && k != "RU" && k != "TT" {
            meta_str.push_str(&format!("{}: {}\n", k, v));
        }
    }
    Paragraph::new(meta_str)
        .block(Block::default().title("Metadata").borders(Borders::ALL))
}

fn setup_terminal() -> io::Result<ratatui::Terminal<CrosstermBackend<Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(ratatui::Terminal::new(backend)?)
}

fn restore_terminal(terminal: &mut ratatui::Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}
