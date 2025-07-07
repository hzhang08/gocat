use crate::game::{GoGame, Stone};
use crate::sgf_parser::sgf_to_string;
use crate::ui_mode_actions::handle_edit_triangles_input;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::io::{self, Stdout};

pub enum UiMode {
    Normal,
    GotoMoveInput { input: String },
    HotkeyHelp,
    ModifyMoveInput { input: String },
    SearchCoordInput { input: String },
    EditCommentInput { input: String },
    EditTrianglesInput { input: String },
    InsertMoveInput { input: String, color: crate::sgf_parser::Player },
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
                    let block = Block::default().title("Goto Move").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow));
                    let text = Paragraph::new(format!("Enter move #: {}", input)).style(Style::default().fg(Color::Yellow)).block(block);
                    f.render_widget(text, area);
                }
                UiMode::HotkeyHelp => {
                    let area = centered_rect(50, 40, size);
                    let block = Block::default().title("Hotkey Help").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow));
                    let help = [
                        "q         Quit",
                        "n / →     Next move",
                        "p / ←     Previous move",
                        "]         Next commented move",
                        "[         Previous commented move",
                        "g         Goto move number",
                        "m         Modify current move",
                        "/         Search for coordinate",
                        "c         Add/Edit move comment",
                        "t         Add/Edit triangles",
                        "i         Insert new move",
                        "x         Remove current move",
                        "h         Show this help",
                        "Esc/Enter Close this help",
                    ].join("\n");
                    let text = Paragraph::new(help).style(Style::default().fg(Color::Yellow)).block(block);
                    f.render_widget(text, area);
                }
                UiMode::ModifyMoveInput { input } => {
                    let area = centered_rect(30, 10, size);
                    let block = Block::default().title("Modify Move").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow));
                    let text = Paragraph::new(format!("Enter coords (e.g., dd): {}", input)).block(block);
                    f.render_widget(text, area);
                }
                UiMode::EditCommentInput { input } => {
                    let area = centered_rect(50, 10, size);
                    let block = Block::default().title("Edit Comment").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow));
                    let text = Paragraph::new(format!("Edit comment: {}", input)).style(Style::default().fg(Color::Yellow)).block(block);
                    f.render_widget(text, area);
                }
                UiMode::EditTrianglesInput { input } => {
                    let area = centered_rect(60, 10, size);
                    let block = Block::default().title("Edit Triangles").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow));
                    let text = Paragraph::new(format!("Comma-separated coords (e.g., dd,ee,fg): {}", input)).style(Style::default().fg(Color::Yellow)).block(block);
                    f.render_widget(text, area);
                }
                UiMode::SearchCoordInput { input } => {
                    let area = centered_rect(30, 10, size);
                    let block = Block::default().title("Search Coord").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow));
                    let text = Paragraph::new(format!("Enter coords (e.g., dd): {}", input)).block(block);
                    f.render_widget(text, area);
                }
                UiMode::InsertMoveInput { input, color } => {
                    let area = centered_rect(40, 12, size);
                    let block = Block::default().title("Insert Move").borders(Borders::ALL).border_style(Style::default().fg(Color::Yellow));
                    let color_str = match color {
                        crate::sgf_parser::Player::Black => "Black",
                        crate::sgf_parser::Player::White => "White",
                    };
                    let text = Paragraph::new(format!("Enter coords (e.g., dd): {}\nColor: {} (Tab to toggle, Enter to confirm)", input, color_str)).style(Style::default().fg(Color::Yellow)).block(block);
                    f.render_widget(text, area);
                }
                _ => {}
            }
        })?;
        if event::poll(std::time::Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let mode_ref = &mut mode;
                    match mode_ref {
                        UiMode::InsertMoveInput { input, color } => {
                            if let Some(new_mode) = crate::ui_mode_actions::handle_insert_move_input(&key, input, color, game) {
                                *mode_ref = new_mode;
                            }
                        },
                        UiMode::Normal => {
                            match crate::ui_mode_actions::handle_normal_input(&key, game) {
                                crate::ui_mode_actions::UiAction::Quit => break,
                                crate::ui_mode_actions::UiAction::ChangeMode(new_mode) => *mode_ref = new_mode,
                                crate::ui_mode_actions::UiAction::Continue => {}
                            }
                        },
                        UiMode::GotoMoveInput { input } => {
                            if let Some(new_mode) = crate::ui_mode_actions::handle_goto_move_input(&key, input, game) {
                                *mode_ref = new_mode;
                            }
                        },
                        UiMode::HotkeyHelp => match key.code {
                            KeyCode::Esc | KeyCode::Enter => *mode_ref = UiMode::Normal,
                            _ => {}
                        },
                        UiMode::SearchCoordInput { input } => {
                            if let Some(new_mode) = crate::ui_mode_actions::handle_search_coord_input(&key, input, game) {
                                *mode_ref = new_mode;
                            }
                        },
                        UiMode::EditTrianglesInput { input } => {
                            if let Some(new_mode) = handle_edit_triangles_input(&key, input, game) {
                                *mode_ref = new_mode;
                            }
                        },
                        UiMode::EditCommentInput { input } => {
                            if let Some(new_mode) = crate::ui_mode_actions::handle_edit_comment_input(&key, input, game) {
                                *mode_ref = new_mode;
                            }
                        },
                        UiMode::ModifyMoveInput { input } => {
                            if let Some(new_mode) = crate::ui_mode_actions::handle_modify_move_input(&key, input, game) {
                                *mode_ref = new_mode;
                            }
                        },
                    }
                },
                _ => {}
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
    use ratatui::text::{Span, Line, Text};
    let mut lines = vec![
        Line::from(vec![Span::styled(
            "Press 'h' to see all available commands.",
            ratatui::style::Style::default().fg(ratatui::style::Color::Yellow)
        )]),
    ];
    let info_str = format!(
        "Move: {}{} / {} | Current Player: {}\n{}",
        move_num, coord_str, total_moves, player, comment_str
    );
    for l in info_str.lines() {
        lines.push(Line::raw(l.to_owned()));
    }
    for (k, v) in &game.metadata {
        if k != "FF"
            && k != "AP"
            && k != "GM"
            && k != "HA"
            && k != "KM"
            && k != "RL"
            && k != "RN"
            && k != "TC"
            && k != "TM"
            && k != "RU"
            && k != "TT"
        {
            lines.push(Line::raw(format!("{}: {}", k, v)));
        }
    }
    Paragraph::new(Text::from(lines))
        .block(Block::default().title("Info").borders(Borders::ALL))

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
