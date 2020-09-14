#![allow(dead_code)]

use crate::response::Package;
use crate::client::{check_pkg, download_pkg};

use crate::util::{
    event::{Event, Events},
    StatefulList,
};

use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};

// in practice, "Downloading" (for showing a bar in tui) is rarely needed
enum PackageState {
    Downloaded,
    Downloading,
    Undownloaded,
}

struct App<'a> {
    options: StatefulList<(&'a str, usize)>,
    packages: StatefulList<(Package, PackageState)>,
    packages_selected: bool,
}

impl<'a> App<'a> {
    fn new(pkgs: Vec<Package>) -> App<'a> {
        App {
            options: StatefulList::with_items(vec![
                ("Start modded", 1),
                ("Start vanilla", 2),
                // these two should actually be "tabs":
                ("Installed", 1),
                ("Online", 3),
                // ("Config Editor", 4),
                // ("Settings", 1),
                // ("Help", 1),
            ]),
            packages: StatefulList::with_items(
                pkgs
                    .iter()
                    .map(|p| {
                        let s = if check_pkg(p.clone()) { PackageState::Downloaded } else { PackageState::Undownloaded };
                        (p.clone(), s)
                    })
                    .collect()
            ),
            packages_selected: false,
        }
    }
}

pub async fn start_app(pkgs: Vec<Package>) -> Result<(), Box<dyn Error>> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    // App
    let mut app = App::new(pkgs);

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
                .split(f.size());

            let block_left = Block::default()
                .title("Risk of Rain 2")
                .borders(Borders::ALL);

            let block_right = Block::default()
                .title("Packages")
                .borders(Borders::ALL);

            let options: Vec<ListItem> = app
                .options
                .items
                .iter()
                .map(|i| {
                    ListItem::new(vec![Spans::from(i.0)])
                })
                .collect();
            let options = List::new(options)
                .block(if app.packages_selected {
                    block_left
                } else {
                    block_left.border_style(Style::default().fg(Color::Cyan))
                })
                .highlight_style(
                    Style::default()
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            f.render_stateful_widget(options, chunks[0], &mut app.options.state);

            let pkgs: Vec<ListItem> = app
                .packages
                .items
                .iter()
                .enumerate()
                .map(|(i, (p, state))| {
                    let mut lines = vec![Spans::from(format!("{} by {}", p.name, p.owner))];

                    if app.packages_selected {
                        if let Some(cur) = app.packages.state.selected() {
                            if cur == i {
                                let dialog_text = match state {
                                    PackageState::Downloaded => "> Downloaded <",
                                    PackageState::Downloading => "...",
                                    PackageState::Undownloaded => "Download? [enter]",
                                };
                                lines.push(Spans::from(Span::styled(
                                    dialog_text,
                                    Style::default().add_modifier(Modifier::BOLD),
                                )));
                            }
                        }
                    }

                    ListItem::new(lines)
                })
                .collect();
            let pkg_list = List::new(pkgs)
                .block(if app.packages_selected {
                    block_right.border_style(Style::default().fg(Color::Red))
                } else {
                    block_right
                })
                .highlight_style(
                    Style::default()
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            f.render_stateful_widget(pkg_list, chunks[1], &mut app.packages.state);
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Left => {
                    if app.packages_selected {
                        app.packages.unselect()
                    } else {
                        app.options.unselect();
                    }
                }
                Key::Down => {
                    if app.packages_selected {
                        app.packages.next()
                    } else {
                        app.options.next();
                    }
                }
                Key::Up => {
                    if app.packages_selected {
                        app.packages.previous()
                    } else {
                        app.options.previous();
                    }
                }
                Key::Right => {
                    app.packages_selected ^= true;
                }
                Key::Char('\n') => {
                    if app.packages_selected {
                        if let Some(i) = app.packages.state.selected() {
                            match app.packages.items[i].1 {
                                PackageState::Undownloaded => {
                                    let pkg = app.packages.items[i].0.clone();
                                    app.packages.items[i].1 = PackageState::Downloading;
                                    download_pkg(pkg).await;
                                    app.packages.items[i].1 = PackageState::Downloaded;
                                },
                                _ => {},
                            };
                        }
                    }
                }
                _ => {}
            },
            _ => {},
        }
    }

    Ok(())
}
