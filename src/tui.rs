//! Modern TUI interface for rfgrep using ratatui
use crate::error::Result as RfgrepResult;
use crate::plugin_system::{EnhancedPluginManager, PluginRegistry};
use crate::processor::SearchMatch;
use crate::search_algorithms::SearchAlgorithm;
use crate::streaming_search::StreamingSearchPipeline;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
    },
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::sync::Arc;

/// TUI application state
#[derive(Debug, Clone)]
pub struct TuiState {
    pub pattern: String,
    pub matches: Vec<SearchMatch>,
    pub current_file_index: usize,
    pub current_match_index: usize,
    pub files: Vec<String>,
    pub search_mode: SearchMode,
    pub algorithm: SearchAlgorithm,
    pub case_sensitive: bool,
    pub context_lines: usize,
    pub show_help: bool,
    pub status_message: String,
    pub search_in_progress: bool,
    pub scroll_offset: usize,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub input_cursor: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Text,
    Word,
    Regex,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Search,
    Command,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            matches: Vec::new(),
            current_file_index: 0,
            current_match_index: 0,
            files: Vec::new(),
            search_mode: SearchMode::Text,
            algorithm: SearchAlgorithm::BoyerMoore,
            case_sensitive: false,
            context_lines: 0,
            show_help: false,
            status_message: "Ready".to_string(),
            search_in_progress: false,
            scroll_offset: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            input_cursor: 0,
        }
    }
}

/// TUI application
pub struct TuiApp {
    pub state: TuiState,
    plugin_manager: Arc<EnhancedPluginManager>,
    streaming_pipeline: Option<StreamingSearchPipeline>,
    list_state: ListState,
    table_state: TableState,
    scrollbar_state: ScrollbarState,
    should_quit: bool,
}

impl TuiApp {
    pub async fn new() -> RfgrepResult<Self> {
        let plugin_manager = Arc::new(EnhancedPluginManager::new());
        let registry = PluginRegistry::new(plugin_manager.clone());

        registry.load_plugins().await?;

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Ok(Self {
            state: TuiState::default(),
            plugin_manager,
            streaming_pipeline: None,
            list_state,
            table_state,
            scrollbar_state: ScrollbarState::default(),
            should_quit: false,
        })
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> RfgrepResult<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if self.handle_key_event(key).await? {
                    break;
                }
            }
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3), // Status bar
            ])
            .split(f.area());

        self.render_header(f, chunks[0]);
        self.render_main_content(f, chunks[1]);
        self.render_status_bar(f, chunks[2]);

        if self.state.show_help {
            self.render_help_overlay(f);
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let header_text = if self.state.input_mode == InputMode::Search {
            format!("rfgrep TUI - Search: {}_", self.state.input_buffer)
        } else if self.state.search_in_progress {
            format!("rfgrep TUI - Searching for: '{}'...", self.state.pattern)
        } else {
            format!("rfgrep TUI - Pattern: '{}'", self.state.pattern)
        };

        let header = Paragraph::new(header_text)
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("rfgrep"));

        f.render_widget(header, area);
    }

    fn render_main_content(&mut self, f: &mut Frame, area: Rect) {
        if self.state.matches.is_empty() {
            self.render_empty_state(f, area);
        } else {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
                .split(area);

            self.render_file_list(f, chunks[0]);
            self.render_matches_table(f, chunks[1]);
        }
    }

    fn render_empty_state(&self, f: &mut Frame, area: Rect) {
        let empty_text = if self.state.pattern.is_empty() {
            "Enter a search pattern to begin..."
        } else {
            "No matches found. Try a different pattern or press 'h' for help."
        };

        let empty_para = Paragraph::new(empty_text)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(empty_para, area);
    }

    fn render_file_list(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .state
            .files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let style = if i == self.state.current_file_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(Span::styled(file.as_str(), style)))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Files"))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_matches_table(&mut self, f: &mut Frame, area: Rect) {
        let matches = &self.state.matches;
        let start_idx = self.state.scroll_offset;
        let end_idx = (start_idx + (area.height as usize).saturating_sub(2)).min(matches.len());

        let rows: Vec<Row> = matches[start_idx..end_idx]
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let line_num = format!("{:<4}", m.line_number);
                let content = if m.line.len() > 80 {
                    format!("{}...", &m.line[..77])
                } else {
                    m.line.clone()
                };

                let style = if start_idx + i == self.state.current_match_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                Row::new(vec![
                    Cell::from(Span::styled(line_num, style)),
                    Cell::from(Span::styled(content, style)),
                ])
            })
            .collect();

        let table = Table::new(rows, &[Constraint::Length(6), Constraint::Min(0)])
            .block(Block::default().borders(Borders::ALL).title("Matches"))
            .column_spacing(1);

        f.render_stateful_widget(table, area, &mut self.table_state);

        if matches.len() > (area.height as usize).saturating_sub(2) {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            f.render_stateful_widget(
                scrollbar,
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut self.scrollbar_state,
            );
        }
    }

    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let status_text = if self.state.search_in_progress {
            "Searching...".to_string()
        } else {
            format!(
                "Matches: {} | Files: {} | Mode: {:?} | Algorithm: {:?} | Case: {}",
                self.state.matches.len(),
                self.state.files.len(),
                self.state.search_mode,
                self.state.algorithm,
                if self.state.case_sensitive {
                    "ON"
                } else {
                    "OFF"
                }
            )
        };

        let status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray))
            .alignment(Alignment::Left);

        f.render_widget(status, area);
    }

    fn render_help_overlay(&self, f: &mut Frame) {
        let area = f.area();
        let help_text = vec![
            "rfgrep TUI Help",
            "",
            "Navigation:",
            "  ↑/↓, j/k    - Navigate matches",
            "  ←/→, h/l    - Navigate files",
            "  Page Up/Dn  - Scroll matches",
            "",
            "Search:",
            "  /           - Enter search input mode",
            "  n           - Next match",
            "  N           - Previous match",
            "  Enter       - Open file in editor",
            "",
            "Search Input Mode:",
            "  Type pattern - Enter search pattern",
            "  Enter       - Execute search",
            "  Esc         - Cancel search",
            "  ←/→         - Move cursor",
            "  Home/End    - Jump to start/end",
            "",
            "Settings:",
            "  c           - Toggle case sensitivity",
            "  m           - Change search mode",
            "  a           - Change algorithm",
            "  r           - Refresh search",
            "",
            "Other:",
            "  h           - Toggle this help",
            "  q           - Quit",
            "  ESC         - Exit current mode",
        ];

        let help_para = Paragraph::new(Text::from(help_text.join("\n")))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title("Help"));

        let centered_area = centered_rect(60, 70, area);
        f.render_widget(Clear, area);
        f.render_widget(help_para, centered_area);
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> RfgrepResult<bool> {
        if self.state.show_help {
            if key.code == KeyCode::Char('h') || key.code == KeyCode::Esc {
                self.state.show_help = false;
            }
            return Ok(false);
        }

        // Handle input modes
        if self.state.input_mode != InputMode::Normal {
            return self.handle_input_mode(key).await;
        }

        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('h') => {
                self.state.show_help = true;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.next_match();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.previous_match();
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.next_file();
            }
            KeyCode::Left => {
                self.previous_file();
            }
            KeyCode::Char('n') => {
                self.next_match();
            }
            KeyCode::Char('N') => {
                self.previous_match();
            }
            KeyCode::Char('c') => {
                self.state.case_sensitive = !self.state.case_sensitive;
                self.state.status_message = format!(
                    "Case sensitivity: {}",
                    if self.state.case_sensitive {
                        "ON"
                    } else {
                        "OFF"
                    }
                );
            }
            KeyCode::Char('m') => {
                self.cycle_search_mode();
            }
            KeyCode::Char('a') => {
                self.cycle_algorithm();
            }
            KeyCode::Char('r') => {
                self.refresh_search().await?;
            }
            KeyCode::Char('/') => {
                self.enter_search_input_mode();
            }
            KeyCode::Enter => {
                self.open_current_file();
            }
            KeyCode::PageUp => {
                self.scroll_up();
            }
            KeyCode::PageDown => {
                self.scroll_down();
            }
            _ => {}
        }

        Ok(false)
    }

    fn next_match(&mut self) {
        if !self.state.matches.is_empty() {
            self.state.current_match_index =
                (self.state.current_match_index + 1) % self.state.matches.len();
            self.update_file_index_from_match();
        }
    }

    fn previous_match(&mut self) {
        if !self.state.matches.is_empty() {
            self.state.current_match_index = if self.state.current_match_index == 0 {
                self.state.matches.len() - 1
            } else {
                self.state.current_match_index - 1
            };
            self.update_file_index_from_match();
        }
    }

    fn next_file(&mut self) {
        if !self.state.files.is_empty() {
            self.state.current_file_index =
                (self.state.current_file_index + 1) % self.state.files.len();
            self.update_match_index_from_file();
        }
    }

    fn previous_file(&mut self) {
        if !self.state.files.is_empty() {
            self.state.current_file_index = if self.state.current_file_index == 0 {
                self.state.files.len() - 1
            } else {
                self.state.current_file_index - 1
            };
            self.update_match_index_from_file();
        }
    }

    fn update_file_index_from_match(&mut self) {
        if let Some(current_match) = self.state.matches.get(self.state.current_match_index) {
            if let Some(file_index) = self
                .state
                .files
                .iter()
                .position(|f| f == &current_match.path.to_string_lossy())
            {
                self.state.current_file_index = file_index;
            }
        }
    }

    fn update_match_index_from_file(&mut self) {
        if let Some(current_file) = self.state.files.get(self.state.current_file_index) {
            if let Some(match_index) = self
                .state
                .matches
                .iter()
                .position(|m| m.path.to_string_lossy() == *current_file)
            {
                self.state.current_match_index = match_index;
            }
        }
    }

    fn cycle_search_mode(&mut self) {
        self.state.search_mode = match self.state.search_mode {
            SearchMode::Text => SearchMode::Word,
            SearchMode::Word => SearchMode::Regex,
            SearchMode::Regex => SearchMode::Text,
        };
        self.state.status_message = format!("Search mode: {:?}", self.state.search_mode);
    }

    fn cycle_algorithm(&mut self) {
        self.state.algorithm = match self.state.algorithm {
            SearchAlgorithm::BoyerMoore => SearchAlgorithm::Regex,
            SearchAlgorithm::Regex => SearchAlgorithm::Simple,
            SearchAlgorithm::Simple => SearchAlgorithm::Simd,
            SearchAlgorithm::Simd => SearchAlgorithm::BoyerMoore,
        };
        self.state.status_message = format!("Algorithm: {:?}", self.state.algorithm);
    }

    fn scroll_up(&mut self) {
        if self.state.scroll_offset > 0 {
            self.state.scroll_offset -= 1;
        }
    }

    fn scroll_down(&mut self) {
        let max_scroll = self.state.matches.len().saturating_sub(10);
        if self.state.scroll_offset < max_scroll {
            self.state.scroll_offset += 1;
        }
    }

    fn open_current_file(&mut self) {
        if let Some(current_match) = self.state.matches.get(self.state.current_match_index) {
            self.state.status_message = format!("Opening: {}", current_match.path.display());
            let path = &current_match.path;

            let editor = std::env::var("EDITOR").ok();
            let result = if let Some(ed) = editor {
                std::process::Command::new(ed).arg(path).spawn()
            } else if cfg!(target_os = "macos") {
                std::process::Command::new("open").arg(path).spawn()
            } else if cfg!(target_os = "windows") {
                std::process::Command::new("cmd")
                    .args(["/C", "start", "", &path.to_string_lossy()])
                    .spawn()
            } else {
                std::process::Command::new("xdg-open").arg(path).spawn()
            };
            if result.is_err() {
                self.state.status_message = format!("Failed to open: {}", path.display());
            }
        }
    }

    async fn refresh_search(&mut self) -> RfgrepResult<()> {
        if self.state.pattern.is_empty() {
            self.state.status_message = "No pattern to search".to_string();
            return Ok(());
        }

        self.state.search_in_progress = true;
        self.state.status_message = "Searching...".to_string();

        use crate::walker::walk_dir;
        use std::path::Path;

        let pattern = self.state.pattern.clone();
        let mut all_matches: Vec<SearchMatch> = Vec::new();
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        let entries: Vec<_> = walk_dir(Path::new(&cwd), true, false).collect();
        for entry in entries {
            let path = entry.path();
            if path.is_file() {
                let res = self.plugin_manager.search_file(path, &pattern).await;
                if let Ok(mut matches) = res {
                    all_matches.append(&mut matches);
                }
            }
        }

        self.set_matches(all_matches);

        self.state.search_in_progress = false;
        self.state.status_message = "Search completed".to_string();

        Ok(())
    }

    pub fn set_pattern(&mut self, pattern: String) {
        self.state.pattern = pattern;
    }

    pub fn set_matches(&mut self, matches: Vec<SearchMatch>) {
        self.state.matches = matches;
        self.state.current_match_index = 0;
        self.state.current_file_index = 0;
        self.state.scroll_offset = 0;

        let mut files = std::collections::HashSet::new();
        for m in &self.state.matches {
            files.insert(m.path.to_string_lossy().to_string());
        }
        self.state.files = files.into_iter().collect();
        self.state.files.sort();
    }

    /// Enter search input mode
    fn enter_search_input_mode(&mut self) {
        self.state.input_mode = InputMode::Search;
        self.state.input_buffer = self.state.pattern.clone();
        self.state.input_cursor = self.state.input_buffer.len();
        self.state.status_message =
            "Enter search pattern (Enter to search, Esc to cancel)".to_string();
    }

    /// Handle input mode key events
    async fn handle_input_mode(&mut self, key: KeyEvent) -> RfgrepResult<bool> {
        match self.state.input_mode {
            InputMode::Search => self.handle_search_input(key).await,
            InputMode::Command => self.handle_command_input(key).await,
            InputMode::Normal => Ok(false),
        }
    }

    /// Handle search input mode
    async fn handle_search_input(&mut self, key: KeyEvent) -> RfgrepResult<bool> {
        match key.code {
            KeyCode::Enter => {
                // Apply the search
                self.state.pattern = self.state.input_buffer.clone();
                self.state.input_mode = InputMode::Normal;
                self.state.input_buffer.clear();
                self.state.input_cursor = 0;
                self.state.status_message = "Searching...".to_string();
                self.refresh_search().await?;
                Ok(false)
            }
            KeyCode::Esc => {
                // Cancel search input
                self.state.input_mode = InputMode::Normal;
                self.state.input_buffer.clear();
                self.state.input_cursor = 0;
                self.state.status_message = "Search cancelled".to_string();
                Ok(false)
            }
            KeyCode::Backspace => {
                if self.state.input_cursor > 0 {
                    self.state.input_cursor -= 1;
                    self.state.input_buffer.remove(self.state.input_cursor);
                }
                Ok(false)
            }
            KeyCode::Delete => {
                if self.state.input_cursor < self.state.input_buffer.len() {
                    self.state.input_buffer.remove(self.state.input_cursor);
                }
                Ok(false)
            }
            KeyCode::Left => {
                if self.state.input_cursor > 0 {
                    self.state.input_cursor -= 1;
                }
                Ok(false)
            }
            KeyCode::Right => {
                if self.state.input_cursor < self.state.input_buffer.len() {
                    self.state.input_cursor += 1;
                }
                Ok(false)
            }
            KeyCode::Home => {
                self.state.input_cursor = 0;
                Ok(false)
            }
            KeyCode::End => {
                self.state.input_cursor = self.state.input_buffer.len();
                Ok(false)
            }
            KeyCode::Char(c) => {
                self.state.input_buffer.insert(self.state.input_cursor, c);
                self.state.input_cursor += 1;
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Handle command input mode (for future use)
    async fn handle_command_input(&mut self, key: KeyEvent) -> RfgrepResult<bool> {
        match key.code {
            KeyCode::Enter => {
                // Process command
                self.state.input_mode = InputMode::Normal;
                self.state.input_buffer.clear();
                self.state.input_cursor = 0;
                self.state.status_message = "Command processed".to_string();
                Ok(false)
            }
            KeyCode::Esc => {
                self.state.input_mode = InputMode::Normal;
                self.state.input_buffer.clear();
                self.state.input_cursor = 0;
                self.state.status_message = "Command cancelled".to_string();
                Ok(false)
            }
            _ => {
                // Handle other keys similar to search input
                self.handle_search_input(key).await
            }
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn init_terminal() -> RfgrepResult<Terminal<CrosstermBackend<Stdout>>> {
    if !is_terminal::is_terminal(&std::io::stdout()) {
        return Err(crate::error::RfgrepError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "TUI requires an interactive terminal. Please run in a proper terminal environment.",
        )));
    }

    if std::env::var("TERM").is_err() {
        return Err(crate::error::RfgrepError::Io(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "TUI requires a proper terminal environment. TERM environment variable not set.",
        )));
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> RfgrepResult<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
