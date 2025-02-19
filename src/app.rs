//! App for ymir

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph, Row,
        StatefulWidget, Table, Widget, Wrap,
    },
    DefaultTerminal,
};

use ratatui::style::palette::tailwind::{CYAN, NEUTRAL, RED, SLATE};
use tokei::LanguageType;

use crate::{
    projects::Project,
    sorting::{Filter, Sorting},
};

#[allow(clippy::struct_excessive_bools)]
pub struct App {
    should_exit: bool,
    show_project_info: bool,
    show_languages: bool,
    projects_list: ProjectsList,
    sort_type: Sorting,
    filter_type: Filter,
    invert: bool,
    git_name: String,

    // Search
    search_text: Option<String>,
    search_index: usize,
    search_count: usize,
}

const SELECTED_STYLE: Style = Style::new().bg(NEUTRAL.c900).add_modifier(Modifier::BOLD);
const INACTIVE_COLOR: Color = RED.c700;
pub const TEXT_FG_COLOR: Color = SLATE.c200;

impl App {
    /// Create a new app with the given list of projects
    pub fn new(projects_list: Vec<Project>) -> Self {
        Self {
            should_exit: false,
            show_project_info: true,
            show_languages: true,
            sort_type: Sorting::Name,
            filter_type: Filter::All,
            projects_list: ProjectsList::from_iter(projects_list),
            invert: false,
            git_name: git2::Config::open_default().map_or(String::new(), |v| {
                v.get_string("user.name").unwrap_or_default()
            }),
            search_text: None,
            search_index: 0,
            search_count: 0,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                if self.search_text.is_some() {
                    self.handle_search_key(key);
                } else {
                    self.handle_key(key);
                }
            };
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != event::KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            // Movement
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('d') => self.select_next_10(),
            KeyCode::Char('u') => self.select_previous_10(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),

            // Toggle
            KeyCode::Char('1') => self.show_project_info = !self.show_project_info,
            KeyCode::Char('2') => self.show_languages = !self.show_languages,

            // Sorting
            KeyCode::Char('h') | KeyCode::Left => {
                self.sort_type = self.sort_type.previous();
                self.projects_list
                    .sort_projects(&self.sort_type, self.invert);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.sort_type = self.sort_type.next();
                self.projects_list
                    .sort_projects(&self.sort_type, self.invert);
            }
            KeyCode::Char('i') => {
                self.invert = !self.invert;
                self.projects_list
                    .sort_projects(&self.sort_type, self.invert);
            }

            // Filtering
            KeyCode::Char('y') => {
                self.filter_type = self.filter_type.previous();
                self.projects_list
                    .filter_projects(&self.filter_type, &self.git_name);
            }
            KeyCode::Char('o') => {
                self.filter_type = self.filter_type.next();
                self.projects_list
                    .filter_projects(&self.filter_type, &self.git_name);
            }

            // Searching
            KeyCode::Char('/') => {
                if self.search_text.is_some() {
                    self.search_text = None;
                } else {
                    self.search_text = Some(String::new());
                }
            }

            _ => {}
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        if key.kind != event::KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Esc => {
                self.search_text = None;
                self.search_index = 0;
            }
            KeyCode::Char(c) => {
                if let Some(v) = self.search_text.as_mut() {
                    v.push(c);
                }
                self.search_count = self.projects_list.search(
                    &self.search_text.clone().unwrap_or_default(),
                    self.search_index,
                );
            }
            KeyCode::Backspace => {
                if let Some(v) = self.search_text.as_mut() {
                    v.pop();
                }
                self.search_count = self.projects_list.search(
                    &self.search_text.clone().unwrap_or_default(),
                    self.search_index,
                );
            }
            KeyCode::Enter => {
                self.search_count = self.projects_list.search(
                    &self.search_text.clone().unwrap_or_default(),
                    self.search_index,
                );

                if self.search_index >= self.search_count.wrapping_sub(1) {
                    self.search_index = 0;
                } else {
                    self.search_index += 1;
                }
            }
            _ => {}
        }
    }

    fn select_next(&mut self) {
        self.projects_list.state.select_next();
    }

    fn select_previous(&mut self) {
        self.projects_list.state.select_previous();
    }

    fn select_next_10(&mut self) {
        self.projects_list.state.select(Some(
            self.projects_list.state.selected().map_or(0, |v| v + 10),
        ));
    }

    fn select_previous_10(&mut self) {
        self.projects_list.state.select(Some(
            self.projects_list
                .state
                .selected()
                .map_or(self.projects_list.items.len(), |v| v.saturating_sub(10)),
        ));
    }

    fn select_first(&mut self) {
        self.projects_list.state.select_first();
    }

    fn select_last(&mut self) {
        self.projects_list.state.select_last();
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, data_area] = if self.show_project_info || self.show_languages {
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area)
        } else {
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(0)]).areas(main_area)
        };

        let [list_area, search_area] = if self.search_text.is_some() {
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).areas(list_area)
        } else {
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(0)]).areas(list_area)
        };

        let [info_area, langs_area] = if self.show_project_info && self.show_languages {
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(data_area)
        } else if !self.show_project_info && self.show_languages {
            Layout::vertical([Constraint::Fill(0), Constraint::Fill(1)]).areas(data_area)
        } else if self.show_project_info && !self.show_languages {
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(0)]).areas(data_area)
        } else {
            Layout::vertical([Constraint::Fill(0), Constraint::Fill(0)]).areas(data_area)
        };

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);

        if self.show_project_info {
            self.render_project_info(info_area, buf);
        }

        if self.search_text.is_some() {
            self.render_search(search_area, buf);
        }

        if self.show_languages {
            self.render_project_langs(langs_area, buf);
        }
    }
}

impl App {
    pub fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Ymir project finder")
            .bold()
            .centered()
            .render(area, buf);
    }

    pub fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let sort_title = vec![
            Span::styled(" <h ", Style::default().fg(CYAN.c500)),
            Span::from(self.sort_type.to_string()),
            Span::styled(" l> ", Style::default().fg(CYAN.c500)),
        ];

        let filter_title = vec![
            Span::styled(" <y ", Style::default().fg(CYAN.c500)),
            Span::from(self.filter_type.to_string()),
            Span::styled(" o> ", Style::default().fg(CYAN.c500)),
        ];

        let mut invert_title = Line::from(vec![
            Span::styled(" i", Style::default().fg(CYAN.c500)),
            Span::from("nvert "),
        ])
        .right_aligned();

        if self.invert {
            invert_title = invert_title.add_modifier(Modifier::BOLD);
        }

        let block = Block::new()
            .title(
                Line::raw(format!("Projects ({})", self.projects_list.items.len())).left_aligned(),
            )
            .title(invert_title)
            .title(Line::from(filter_title).right_aligned())
            .title(Line::from(sort_title).right_aligned())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        let items: Vec<ListItem> = self
            .projects_list
            .items
            .iter()
            .map(ListItem::from)
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.projects_list.state);
    }

    fn render_search(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(
                Line::from(format!("[{}/{}]", self.search_index + 1, self.search_count))
                    .right_aligned(),
            )
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED);

        Paragraph::new(self.search_text.as_ref().map_or("", |v| v))
            .block(block)
            .render(area, buf);
    }

    fn render_project_info(&self, area: Rect, buf: &mut Buffer) {
        let info = self.projects_list.state.selected().map_or_else(
            || "Nothing selected...".to_string(),
            |i| self.projects_list.items[i].to_string(),
        );

        let title = vec![
            Span::from("["),
            Span::styled("1", Style::default().fg(CYAN.c500)),
            Span::from("] Project Info"),
        ];

        let block = Block::new()
            .title(Line::from(title).left_aligned())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .padding(Padding::horizontal(1));

        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }

    fn render_project_langs(&self, area: Rect, buf: &mut Buffer) {
        let mut total_files = 0;
        let mut total_lines = 0;
        let mut total_code = 0;
        let mut total_comments = 0;
        let mut total_blanks = 0;

        let rows: Vec<Row> = self
            .projects_list
            .state
            .selected()
            .map_or_else(Vec::new, |i| {
                self.projects_list.items[i]
                    .languages
                    .iter()
                    .map(|(ltype, l)| {
                        total_files += l.files;
                        total_lines += l.lines;
                        total_code += l.code;
                        total_comments += l.comments;
                        total_blanks += l.blanks;

                        Row::new(vec![
                            LanguageType::list()
                                .get(*ltype as usize)
                                .map_or("Error".to_string(), ToString::to_string),
                            l.files.to_string(),
                            l.lines.to_string(),
                            l.code.to_string(),
                            l.comments.to_string(),
                            l.blanks.to_string(),
                        ])
                    })
                    .collect::<Vec<Row>>()
            });

        let header = ["Language", "Files", "Lines", "Code", "Comments", "Blanks"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .height(1);

        let footer = [
            "Total".to_string(),
            total_files.to_string(),
            total_lines.to_string(),
            total_code.to_string(),
            total_comments.to_string(),
            total_blanks.to_string(),
        ]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .height(1);

        let title = vec![
            Span::from("["),
            Span::styled("2", Style::default().fg(CYAN.c500)),
            Span::from("] Languages"),
        ];

        let block = Block::new()
            .title(Line::from(title).left_aligned())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .padding(Padding::horizontal(1));

        Widget::render(
            Table::new(
                rows,
                [
                    Constraint::Percentage(25),
                    Constraint::Percentage(15),
                    Constraint::Percentage(15),
                    Constraint::Percentage(15),
                    Constraint::Percentage(15),
                    Constraint::Percentage(15),
                ],
            )
            .header(header)
            .footer(footer)
            .block(block),
            area,
            buf,
        );
    }
}

pub fn get_remote_username(project: &Project) -> String {
    project
        .git_info
        .remote_url
        .as_ref()
        .map_or("", |v| v.split('/').nth(3).unwrap_or_default())
        .to_string()
}

struct ProjectsList {
    items: Vec<Project>,
    items_state: Vec<Project>,
    state: ListState,
}

impl ProjectsList {
    fn sort_projects(&mut self, sort_type: &Sorting, invert: bool) {
        let mut items: Vec<Project> = self.items.clone();

        match sort_type {
            Sorting::Name => {
                items.sort_by(|a, b| a.path.cmp(&b.path));
            }
            Sorting::Size => {
                items.sort_by(|a, b| a.size.cmp(&b.size));
            }
            Sorting::Commits => {
                items.sort_by(|a, b| a.git_info.commit_count.cmp(&b.git_info.commit_count));
            }
            Sorting::CreationDate => {
                items.sort_by(|a, b| a.git_info.init_date.cmp(&b.git_info.init_date));
            }
            Sorting::ModificationDate => {
                items.sort_by(|a, b| {
                    a.git_info
                        .last_commit_date
                        .cmp(&b.git_info.last_commit_date)
                });
            }
            Sorting::Loc => {
                items.sort_by(|a, b| a.languages_total.lines.cmp(&b.languages_total.lines));
            }
        }

        if invert {
            items.reverse();
        }

        self.items = items;
        self.state.select(Some(0));
    }

    fn filter_projects(&mut self, filter_type: &Filter, username: &str) {
        let items = self.items_state.clone();

        let items = match filter_type {
            Filter::All => items,
            Filter::Owned => items
                .into_iter()
                .filter(|v| get_remote_username(v) == username)
                .collect(),
            Filter::NotOwned => items
                .into_iter()
                .filter(|v| get_remote_username(v) != username)
                .collect(),
            Filter::HasRemote => items
                .into_iter()
                .filter(|v| v.git_info.remote_url.is_some())
                .collect(),
            Filter::NoRemote => items
                .into_iter()
                .filter(|v| v.git_info.remote_url.is_none())
                .collect(),
        };

        self.items = items;
        if self.items.is_empty() {
            self.state.select(None);
        } else {
            self.state.select(Some(0));
        }
    }

    fn search(&mut self, search_text: &str, index: usize) -> usize {
        let filtered_indices: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, p)| p.path.to_string_lossy().to_string().contains(search_text))
            .map(|(idx, _)| idx)
            .collect();

        if let Some(selected_idx) = filtered_indices.get(index) {
            self.state.select(Some(*selected_idx));
        } else {
            self.state.select(None);
        }

        filtered_indices.len()
    }
}

impl FromIterator<Project> for ProjectsList {
    fn from_iter<I: IntoIterator<Item = Project>>(iter: I) -> Self {
        let state = ListState::default();
        let items: Vec<Project> = iter.into_iter().collect();
        Self {
            items: items.clone(),
            items_state: items,
            state,
        }
    }
}

impl From<&Project> for ListItem<'_> {
    fn from(value: &Project) -> Self {
        let mut item = ListItem::new(value.path.display().to_string());

        if value.git_info.commit_count == 0 {
            item = item.fg(INACTIVE_COLOR);
        }

        item
    }
}
