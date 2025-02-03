//! App for ymir
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::Line,
    widgets::{
        Block, Borders, Cell, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph, Row,
        StatefulWidget, Table, Widget, Wrap,
    },
    DefaultTerminal,
};

use ratatui::style::palette::tailwind::{NEUTRAL, RED, SLATE};

use crate::projects::Project;

pub struct App {
    should_exit: bool,
    show_project_info: bool,
    show_languages: bool,
    projects_list: ProjectsList,
}

const SELECTED_STYLE: Style = Style::new().bg(NEUTRAL.c900).add_modifier(Modifier::BOLD);
const INACTIVE_COLOR: Color = RED.c700;
const TEXT_FG_COLOR: Color = SLATE.c200;

impl App {
    /// Create a new app with the given list of projects
    pub fn new(projects_list: Vec<Project>) -> Self {
        Self {
            should_exit: false,
            show_project_info: true,
            show_languages: true,
            projects_list: ProjectsList::from_iter(projects_list),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
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
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('d') => self.select_next_10(),
            KeyCode::Char('u') => self.select_previous_10(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('1') => self.show_project_info = !self.show_project_info,
            KeyCode::Char('2') => self.show_languages = !self.show_languages,
            _ => {}
        }
    }

    fn select_none(&mut self) {
        self.projects_list.state.select(None);
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

        if self.show_languages {
            self.render_project_langs(langs_area, buf);
        }
    }
}

impl App {
    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Ymir project finder")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Use ↓↑ to move, ← to unselect, g/G to go top/bottom.")
            .centered()
            .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Projects").left_aligned())
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

    fn render_project_info(&self, area: Rect, buf: &mut Buffer) {
        let info = self.projects_list.state.selected().map_or_else(
            || "Nothing selected...".to_string(),
            |i| self.projects_list.items[i].to_string(),
        );

        let block = Block::new()
            .title(Line::raw("[1] Project Info").left_aligned())
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
                            ltype.to_string(),
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

        let block = Block::new()
            .title(Line::raw("[2] Languages").left_aligned())
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

struct ProjectsList {
    items: Vec<Project>,
    state: ListState,
}

impl FromIterator<Project> for ProjectsList {
    fn from_iter<I: IntoIterator<Item = Project>>(iter: I) -> Self {
        let state = ListState::default();
        Self {
            items: iter.into_iter().collect(),
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
