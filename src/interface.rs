#![allow(dead_code)]

use crate::response::Package;
use crate::client::download_pkg;
use crate::r2mm;

use crate::util::{
    event::{Event, Events},
    StatefulList,
};

use std::{error::Error, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::{Backend, TermionBackend},
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem},
    Frame, Terminal,
};

// in practice, "Downloading" (for showing a bar in tui) is rarely needed
#[derive(PartialEq)]
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
                        let s = if r2mm::check_pkg(p.clone()) { PackageState::Downloaded } else { PackageState::Undownloaded };
                        (p.clone(), s)
                    })
                    .collect()
            ),
            installed_count: r2mm::count_pkgs().unwrap(),
        }
    }

    async fn on_enter(&mut self) {
        match self.packages.state.selected() {
            Some(i) if self.state == AppWindow::Packages => {
                if self.packages.items[i].1 == PackageState::Undownloaded {
                    let pkg = self.packages.items[i].0.clone();
                    self.packages.items[i].1 = PackageState::Downloading;
                    match download_pkg(pkg, self.packages.items.iter().map(|(p,i)| {p.clone()}).collect()).await {
                        Ok(_) => {
                            self.packages.items[i].1 = PackageState::Downloaded;
                            self.installed_count += 1;
                        }
                        _ => {}
                    }
                }
            }
            Some(_) | None => {}
        }
    }
}

fn create_block(title: &str) -> Block {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
}

fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(f.size());

    let sidebar = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[0]);

    let block_ror2 = create_block("Risk of Rain2");
    let block_profiles = create_block("Mods");
    let block_packages = create_block("Packages");

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

    // Note - hardcoding this for now, since I'm not sure of the best approach to take to fill out
    // package counts for installed / online rows.
    let profiles = vec![
        ListItem::new(vec![
            Spans::from(format!("Installed ({})", app.installed_count)),
        ]),
        ListItem::new(vec![
            Spans::from(format!("Online ({})", app.packages.items.len())),
        ]),
    ];
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

    app.ror2.next();
    app.profiles.next();

    loop {
        terminal.draw(|f| {draw(f, &mut app)})?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Left | Key::Esc => {
                    match app.state {
                        AppWindow::Packages => app.packages.unselect(),
                        _ => {},
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
                    app.on_enter().await;
                }
                _ => {}
            },
            _ => {},
        }
    }

    Ok(())
}
