use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        Block, Clear, List, ListDirection, ListItem, ListState, Paragraph, StatefulWidget, Widget,
    },
    DefaultTerminal, Frame,
};
use ssh_config::config::{SSHConfig, SSHConfigValues};
use std::{fs::read_to_string, io, process::Command};

mod ssh_config;

#[derive(Debug)]
pub struct App {
    state: AppState,
    searchable_strings: Vec<String>,
    ssh_config_values: Vec<SSHConfigValues>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    clear_field: bool,
    exit: ExitState,
    list_state: ListState,
    current_input: String,
    shown_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitState {
    Running,
    Exit,
    Selected,
}

impl App {
    fn with_config(ssh_config: SSHConfig) -> Self {
        let listable_config_values =
            Self::listable_hosts(ssh_config.host_specific_config.into_values());

        App {
            state: AppState {
                exit: ExitState::Running,
                clear_field: false,
                // TODO: Move this out of the state cause this list should not change
                list_state: ListState::default().with_selected(Some(0)),
                current_input: String::with_capacity(50),
                shown_indices: (0..listable_config_values.len()).collect(),
            },
            searchable_strings: listable_config_values
                .iter()
                .map(|value| {
                    format!(
                        "{} {}",
                        value.host.clone().unwrap(),
                        value.hostname.clone().unwrap()
                    )
                })
                .collect(),
            ssh_config_values: listable_config_values,
        }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<Option<SSHConfigValues>> {
        while self.state.exit == ExitState::Running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        match self.state.exit {
            ExitState::Exit => Ok(None),
            ExitState::Selected => Ok(Some(
                self.ssh_config_values.remove(self.state.shown_indices[self
                    .state
                    .list_state
                    .selected()
                    .expect("A value was selected but was not actually selected")]),
            )),
            _ => panic!("We are somehow running after it was stopped"),
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let mut new_state = self.state.clone();
        frame.render_stateful_widget(&*self, frame.area(), &mut new_state);
        self.state = new_state;
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        let has_control = { || key_event.modifiers.contains(KeyModifiers::CONTROL) };

        match key_event.code {
            KeyCode::Esc => self.exit(),
            KeyCode::Up => self.up(),
            KeyCode::Down => self.down(),
            KeyCode::Char('p') if has_control() => self.up(),
            KeyCode::Char('n') if has_control() => self.down(),
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Enter => self.select(),
            KeyCode::Char(ch) => self.add_char(ch),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.state.exit = ExitState::Exit;
    }

    fn select(&mut self) {
        if self.state.list_state.selected().is_some() {
            self.state.exit = ExitState::Selected;
        }
    }

    fn up(&mut self) {
        self.state.list_state.select_next();
    }

    fn down(&mut self) {
        self.state.list_state.select_previous();
    }

    fn delete_char(&mut self) {
        if self.state.current_input.pop().is_some() {
            self.recalculate_shown();
        }
    }

    fn add_char(&mut self, ch: char) {
        self.state.current_input.push(ch);
        self.recalculate_shown();
    }

    fn recalculate_shown(&mut self) {
        // TODO: Should probably do caching cause this is wasteful
        self.state.shown_indices.clear();
        self.state.clear_field = true;
        for (i, str) in self.searchable_strings.iter().enumerate() {
            if str.contains(&self.state.current_input) {
                self.state.shown_indices.push(i);
            }
        }

        if !self.state.shown_indices.is_empty() && self.state.list_state.selected().is_none() {
            self.state.list_state.select_first();
        }
    }

    fn hosts_to_show(&self) -> impl Iterator<Item = &SSHConfigValues> {
        self.state
            .shown_indices
            .iter()
            .map(|idx| &self.ssh_config_values[*idx])
    }

    fn listable_hosts(values: impl Iterator<Item = SSHConfigValues>) -> Vec<SSHConfigValues> {
        values
            .filter(|value| value.host.is_some() && value.hostname.is_some())
            .collect()
    }
}

impl StatefulWidget for &App {
    type State = AppState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut AppState,
    ) {
        let title = Line::from(" Fuzzy SSH Search ".bold());
        let instructions = Line::from(vec![
            " Move Up ".into(),
            "<Up>".blue().bold(),
            " or ".into(),
            "<C-p>".blue().bold(),
            " Move Down ".into(),
            "<Down>".blue().bold(),
            " or ".into(),
            "<C-n>".blue().bold(),
            " Select ".into(),
            "Enter ".blue().bold(),
            " Quit ".into(),
            "<Esc> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let items: Vec<ListItem> = self
            .hosts_to_show()
            .map(|config| HostItem { config }.into())
            .collect();

        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Max(3)]).split(area);

        if state.clear_field {
            Clear.render(layout[0], buf);
            state.clear_field = false;
        }

        StatefulWidget::render(
            List::new(items)
                .direction(ListDirection::BottomToTop)
                .highlight_style(Style::default().on_white().italic().fg(Color::Red))
                .block(block),
            layout[0],
            buf,
            &mut state.list_state,
        );

        InputBox::new(&self.state.current_input).render(layout[1], buf);
    }
}

struct InputBox<'a> {
    current_input: &'a str,
}

impl<'a> InputBox<'a> {
    fn new(current: &'a str) -> Self {
        Self {
            current_input: current,
        }
    }
}

impl Widget for InputBox<'_> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from("Search");
        let block = Block::bordered().title(title.left_aligned());

        let input_line = Line::from(vec!["> ".into(), self.current_input.into()]);

        let text = Text::from(input_line);

        let paragraph = Paragraph::new(text).block(block);

        paragraph.render(area, buf);
    }
}

struct HostItem<'a> {
    config: &'a SSHConfigValues,
}

impl From<HostItem<'_>> for ListItem<'_> {
    fn from(value: HostItem<'_>) -> Self {
        ListItem::from(Line::from(vec![
            value.config.host.clone().unwrap().blue().bold(),
            "       ".into(),
            value.config.hostname.clone().unwrap().light_blue(),
        ]))
    }
}

fn main() -> anyhow::Result<()> {
    let ssh_config_location = shellexpand::tilde("~/.ssh/config");
    let config_file_data = read_to_string(ssh_config_location.to_string())?;

    let config = SSHConfig::from_string(&config_file_data)?;

    let mut terminal = ratatui::init();
    let app_result = App::with_config(config).run(&mut terminal);
    ratatui::restore();
    if let Some(config) = app_result? {
        let host = config.host.expect("We should have a host if we searched for it");

        let status = Command::new("ssh")
            .arg(host)
            .status()?;

        std::process::exit(status.code().unwrap_or_else(|| 1));
    }
    Ok(())
}
