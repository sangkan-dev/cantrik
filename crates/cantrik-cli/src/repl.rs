//! Interactive TUI REPL (Sprint 4): ratatui + crossterm, streaming LLM, slash commands.

use std::collections::VecDeque;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

use cantrik_core::config::AppConfig;
use cantrik_core::llm::{self, LlmError};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tokio::runtime::Handle;

use crate::commands::doctor;

const MAX_THINKING_LINES: usize = 48;
const MAX_INPUT_HISTORY: usize = 100;

#[derive(Debug, Clone)]
enum SlashCmd {
    Cost,
    Memory,
    Doctor,
    Help,
    Exit,
    Unknown(String),
}

#[derive(Debug)]
enum LlmUiMsg {
    Chunk(String),
    Done(Result<(), String>),
}

struct ReplState {
    input: String,
    /// Newest at end; Up arrow walks `hist_cursor`.
    input_history: Vec<String>,
    hist_cursor: Option<usize>,
    thinking: VecDeque<String>,
    /// Full transcript of assistant replies (for tail display).
    output: String,
    busy: bool,
}

impl ReplState {
    fn new() -> Self {
        Self {
            input: String::new(),
            input_history: Vec::new(),
            hist_cursor: None,
            thinking: VecDeque::new(),
            output: String::new(),
            busy: false,
        }
    }

    fn push_thinking(&mut self, line: impl Into<String>) {
        self.thinking.push_back(line.into());
        while self.thinking.len() > MAX_THINKING_LINES {
            self.thinking.pop_front();
        }
    }
}

/// Run interactive REPL; blocks current thread — call from `spawn_blocking` with Tokio handle.
pub fn run_sync(cwd: PathBuf, config: AppConfig, rt: Handle) -> IoResult<()> {
    let mut terminal = ratatui::init();
    let run = run_loop(&mut terminal, cwd, config, rt);
    ratatui::restore();
    run
}

fn run_loop(
    terminal: &mut DefaultTerminal,
    cwd: PathBuf,
    config: AppConfig,
    rt: Handle,
) -> IoResult<()> {
    let mut state = ReplState::new();
    let (tx_llm, rx_llm) = std::sync::mpsc::channel::<LlmUiMsg>();

    state.push_thinking("Cantrik REPL — /help for commands, Ctrl+C to quit.".to_string());

    loop {
        terminal.draw(|frame| ui(frame, &state))?;

        while let Ok(msg) = rx_llm.try_recv() {
            match msg {
                LlmUiMsg::Chunk(s) => {
                    state.output.push_str(&s);
                }
                LlmUiMsg::Done(Ok(())) => {
                    state.busy = false;
                    state.output.push('\n');
                    state.push_thinking("assistant: stream finished OK.".to_string());
                }
                LlmUiMsg::Done(Err(e)) => {
                    state.busy = false;
                    state.push_thinking(format!("assistant error: {e}"));
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(80))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && matches!(key.code, KeyCode::Char('c'))
                    {
                        return Ok(());
                    }

                    if state.busy {
                        continue;
                    }

                    match key.code {
                        KeyCode::Char(c) => {
                            state.input.push(c);
                            state.hist_cursor = None;
                        }
                        KeyCode::Backspace => {
                            state.input.pop();
                            state.hist_cursor = None;
                        }
                        KeyCode::Enter => {
                            let line = std::mem::take(&mut state.input);
                            state.hist_cursor = None;
                            if line.trim().is_empty() {
                                continue;
                            }
                            if !line.starts_with('/') {
                                state.input_history.push(line.clone());
                                if state.input_history.len() > MAX_INPUT_HISTORY {
                                    state.input_history.remove(0);
                                }
                            }
                            if handle_line(&line, cwd.as_path(), &config, &mut state, &rt, &tx_llm)?
                            {
                                return Ok(());
                            }
                        }
                        KeyCode::Up => {
                            if state.input_history.is_empty() {
                                continue;
                            }
                            let idx = state.hist_cursor.unwrap_or(state.input_history.len());
                            if idx > 0 {
                                let next = idx - 1;
                                state.input = state.input_history[next].clone();
                                state.hist_cursor = Some(next);
                            }
                        }
                        KeyCode::Down => {
                            if state.hist_cursor.is_none() {
                                continue;
                            }
                            let idx = state.hist_cursor.unwrap();
                            if idx + 1 < state.input_history.len() {
                                let next = idx + 1;
                                state.input = state.input_history[next].clone();
                                state.hist_cursor = Some(next);
                            } else {
                                state.input.clear();
                                state.hist_cursor = None;
                            }
                        }
                        KeyCode::Esc => return Ok(()),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

fn handle_line(
    line: &str,
    cwd: &Path,
    config: &AppConfig,
    state: &mut ReplState,
    rt: &Handle,
    tx_llm: &std::sync::mpsc::Sender<LlmUiMsg>,
) -> IoResult<bool> {
    let trimmed = line.trim();
    let lower = trimmed.to_ascii_lowercase();
    if matches!(lower.as_str(), "exit" | "quit") {
        return Ok(true);
    }

    if let Some(cmd) = parse_slash(trimmed) {
        match cmd {
            SlashCmd::Exit => return Ok(true),
            SlashCmd::Help => {
                state.push_thinking(
                    "/cost — usage cost (stub) · /memory — tiers · /doctor — health · /exit — quit",
                );
            }
            SlashCmd::Cost => {
                let p = config.llm.provider.as_deref().unwrap_or("(unset)");
                let m = config
                    .llm
                    .model
                    .as_deref()
                    .unwrap_or("(default from providers.toml)");
                state.push_thinking(
                    "/cost: (stub) session & monthly tracking not implemented yet (see Sprint 14)."
                        .to_string(),
                );
                state.push_thinking(format!(
                    "  active target from config: provider={p} model={m}"
                ));
            }
            SlashCmd::Memory => {
                state.push_thinking("Memory tiers (PRD):".to_string());
                state
                    .push_thinking("  Tier1 working: this REPL session (in RAM only).".to_string());
                state.push_thinking(
                    "  Tier2 session / Tier3 project vector / Tier4 global: not persisted yet (Sprint 6–7)."
                        .to_string(),
                );
            }
            SlashCmd::Doctor => {
                state.push_thinking("—— doctor ——".to_string());
                for l in doctor::report_lines(cwd) {
                    state.push_thinking(l);
                }
            }
            SlashCmd::Unknown(name) => {
                state.push_thinking(format!("unknown command /{name} — try /help"));
            }
        }
        return Ok(false);
    }

    // LLM
    let prompt = trimmed.to_string();
    state.busy = true;
    state.push_thinking(format!(
        "thinking: calling LLM… ({})",
        &prompt[..prompt.len().min(80)]
    ));
    let cfg = config.clone();
    let tx_chunk = tx_llm.clone();
    let tx_done = tx_llm.clone();
    rt.spawn(async move {
        let r = llm::ask_stream_chunks(&cfg, &prompt, &mut |s| {
            let _ = tx_chunk.send(LlmUiMsg::Chunk(s.to_string()));
            Ok(())
        })
        .await;
        let _ = tx_done.send(LlmUiMsg::Done(r.map_err(|e: LlmError| e.to_string())));
    });

    Ok(false)
}

fn parse_slash(line: &str) -> Option<SlashCmd> {
    let t = line.trim();
    if !t.starts_with('/') {
        return None;
    }
    let rest = t.strip_prefix('/')?.trim_start();
    let mut parts = rest.split_whitespace();
    let head = parts
        .next()
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    if head.is_empty() {
        return Some(SlashCmd::Help);
    }
    Some(match head.as_str() {
        "cost" => SlashCmd::Cost,
        "memory" => SlashCmd::Memory,
        "doctor" => SlashCmd::Doctor,
        "help" | "?" => SlashCmd::Help,
        "exit" | "quit" => SlashCmd::Exit,
        other => SlashCmd::Unknown(other.to_string()),
    })
}

fn ui(frame: &mut ratatui::Frame, state: &ReplState) {
    let vertical = Layout::vertical([
        Constraint::Length(10),
        Constraint::Min(4),
        Constraint::Length(3),
    ]);
    let [thinking_area, main_area, input_area] = vertical.areas(frame.area());

    let thinking_items: Vec<ListItem> = state
        .thinking
        .iter()
        .rev()
        .take(9)
        .rev()
        .map(|l| {
            ListItem::new(
                ratatui::text::Line::from(l.as_str()).style(Style::default().fg(Color::DarkGray)),
            )
        })
        .collect();
    let think_block = Block::default()
        .borders(Borders::ALL)
        .title(" thinking / log ")
        .border_style(Style::default().fg(Color::Yellow));
    let think = List::new(thinking_items).block(think_block);
    frame.render_widget(think, thinking_area);

    let visible = main_area.height.saturating_sub(2) as usize;
    let main_text = tail_lines(&state.output, visible.max(1));
    let main_block = Block::default()
        .borders(Borders::ALL)
        .title(" assistant (streaming) ")
        .border_style(Style::default().fg(Color::Cyan));
    let main_para = Paragraph::new(main_text).block(main_block);
    frame.render_widget(main_para, main_area);

    let busy = if state.busy { " [busy]" } else { "" };
    let input_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" cantrik>{busy} "))
        .border_style(Style::default().fg(Color::Green));
    let input_para = Paragraph::new(state.input.as_str()).block(input_block);
    frame.render_widget(input_para, input_area);
}

fn tail_lines(s: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    if lines.len() <= max_lines {
        return s.to_string();
    }
    lines[lines.len() - max_lines..].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_slash_cmds() {
        assert!(matches!(parse_slash("/cost"), Some(SlashCmd::Cost)));
        assert!(matches!(parse_slash("/memory"), Some(SlashCmd::Memory)));
        assert!(matches!(parse_slash("/doctor"), Some(SlashCmd::Doctor)));
        assert!(matches!(parse_slash("/exit"), Some(SlashCmd::Exit)));
        assert!(matches!(parse_slash("/help"), Some(SlashCmd::Help)));
        assert!(matches!(
            parse_slash("/nope"),
            Some(SlashCmd::Unknown(x)) if x == "nope"
        ));
        assert!(parse_slash("hello").is_none());
    }
}
