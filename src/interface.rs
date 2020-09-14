#![allow(dead_code)]

use crate::response::Package;
use crate::client::{check_pkg, download_pkg};
use crate::r2mm;

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

#[derive(PartialEq)]
enum AppWindow {
    Ror2,
    Profile,
    // TODO - search should be a small window above packages?
    // Search
    Packages,
}

struct App<'a> {
    state: AppWindow,
    ror2: StatefulList<(&'a str, usize)>,
    profiles: StatefulList<(&'a str, usize)>,
    packages: StatefulList<(Package, PackageState)>,
    installed_count: usize,
}

impl<'a> App<'a> {
    fn new(pkgs: Vec<Package>) -> App<'a> {
        App {
            state: AppWindow::Ror2,
            ror2: StatefulList::with_items(vec![
                ("Start modded", 1),
                ("Start vanilla", 1),
            ]),
            profiles: StatefulList::with_items(vec![
                ("Installed", 1),
                ("Online", 1),
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
            installed_count: r2mm::count_pkgs(),
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

            let sidebar = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(chunks[0]);

            let block_ror2 = Block::default()
                .title("Risk of Rain 2")
                .borders(Borders::ALL);

            let block_profiles = Block::default()
                .title("Mods")
                .borders(Borders::ALL);

            let block_packages = Block::default()
                .title("Packages")
                .borders(Borders::ALL);

            let ror2: Vec<ListItem> = app
                .ror2
                .items
                .iter()
                .map(|i| {
                    ListItem::new(vec![Spans::from(i.0)])
                })
                .collect();
            let ror2 = List::new(ror2)
                .block(if app.state == AppWindow::Ror2 {
                    block_ror2.border_style(Style::default().fg(Color::Cyan))
                } else {
                    block_ror2
                })
                .highlight_style(
                    Style::default()
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            f.render_stateful_widget(ror2, sidebar[0], &mut app.ror2.state);

            let profiles: Vec<ListItem> = app
                .profiles
                .items
                .iter()
                .map(|i| {
                    ListItem::new(vec![Spans::from(i.0)])
                })
                .collect();
            let profiles = List::new(profiles)
                .block(if app.state == AppWindow::Profile {
                    block_profiles.border_style(Style::default().fg(Color::Cyan))
                } else {
                    block_profiles
                })
                .highlight_style(
                    Style::default()
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            f.render_stateful_widget(profiles, sidebar[1], &mut app.profiles.state);

            let pkgs: Vec<ListItem> = app
                .packages
                .items
                .iter()
                .enumerate()
                .map(|(i, (p, state))| {
                    let mut lines = vec![Spans::from(format!("{} by {}", p.name, p.owner))];

                    if app.state == AppWindow::Packages {
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
                .block(if app.state == AppWindow::Packages {
                    block_packages.border_style(Style::default().fg(Color::Red))
                } else {
                    block_packages
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
                    match app.state {
                        AppWindow::Ror2 => app.ror2.unselect(),
                        AppWindow::Profile => app.profiles.unselect(),
                        AppWindow::Packages => app.packages.unselect(),
                    };
                }
                Key::Down => {
                    match app.state {
                        AppWindow::Ror2 => app.ror2.next(),
                        AppWindow::Profile => app.profiles.next(),
                        AppWindow::Packages => app.packages.next(),
                    };
                }
                Key::Up => {
                    match app.state {
                        AppWindow::Ror2 => app.ror2.previous(),
                        AppWindow::Profile => app.profiles.previous(),
                        AppWindow::Packages => app.packages.previous(),
                    };
                }
                Key::Char('\t') => {
                    match app.state {
                        AppWindow::Ror2 => app.state = AppWindow::Profile,
                        AppWindow::Profile => app.state = AppWindow::Packages,
                        AppWindow::Packages => app.state = AppWindow::Ror2,
                    };
                }
                Key::Char('\n') => {
                    if app.state == AppWindow::Packages {
                        if let Some(i) = app.packages.state.selected() {
                            match app.packages.items[i].1 {
                                PackageState::Undownloaded => {
                                    let pkg = app.packages.items[i].0.clone();
                                    app.packages.items[i].1 = PackageState::Downloading;
                                    download_pkg(pkg).await;
                                    app.packages.items[i].1 = PackageState::Downloaded;
                                    app.installed_count += 1;
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
