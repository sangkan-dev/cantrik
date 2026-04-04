//! Interactive TUI REPL (Sprint 4): ratatui + crossterm, streaming LLM, slash commands.

use std::collections::VecDeque;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use cantrik_core::config::{AppConfig, effective_cultural_wisdom, effective_tui_split_pane};
use cantrik_core::cultural_wisdom;
use cantrik_core::llm::{self, LlmError};
use cantrik_core::session::{
    append_message, build_llm_prompt, connect_pool, load_anchors_combined, maybe_summarize_session,
    memory_db_path, message_count, open_or_create_session, session_project_fingerprint,
};
use cantrik_core::usage_ledger::{current_year_month_utc, month_spend_usd, session_spend_usd};
use cantrik_core::visualize::{self, VisualizeKind};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tokio::runtime::Handle;

use crate::commands::agents_cmd;
use crate::commands::doctor;
use crate::commands::plan as plan_cmd;

const MAX_THINKING_LINES: usize = 48;
const MAX_INPUT_HISTORY: usize = 100;

#[derive(Debug, Clone)]
enum SlashCmd {
    Cost,
    Memory,
    Doctor,
    Plan(Option<String>),
    Agents(Option<String>),
    /// Optional tail: callgraph | architecture | dependencies
    Visualize(Option<String>),
    Help,
    Exit,
    Unknown(String),
}

#[derive(Debug)]
enum LlmUiMsg {
    Chunk(String),
    Done(Result<(), String>, String),
}

#[derive(Clone)]
struct ReplMemory {
    pool: cantrik_core::session::SqlitePool,
    session_id: String,
}

struct ReplState {
    input: String,
    /// Newest at end; Up arrow walks `hist_cursor`.
    input_history: Vec<String>,
    hist_cursor: Option<usize>,
    thinking: VecDeque<String>,
    /// Full transcript of assistant replies (for tail display).
    output: String,
    /// Last `/visualize` output (shown in split-pane preview).
    preview: String,
    busy: bool,
    mem: Option<ReplMemory>,
}

impl ReplState {
    fn new(mem: Option<ReplMemory>) -> Self {
        Self {
            input: String::new(),
            input_history: Vec::new(),
            hist_cursor: None,
            thinking: VecDeque::new(),
            output: String::new(),
            preview: String::new(),
            busy: false,
            mem,
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
    let mem = match rt.block_on(connect_pool()) {
        Ok(pool) => match rt.block_on(open_or_create_session(&pool, cwd.as_path())) {
            Ok(session_id) => Some(ReplMemory { pool, session_id }),
            Err(_) => None,
        },
        Err(_) => None,
    };
    let run = run_loop(&mut terminal, cwd, config, rt, mem);
    ratatui::restore();
    run
}

fn run_loop(
    terminal: &mut DefaultTerminal,
    cwd: PathBuf,
    config: AppConfig,
    rt: Handle,
    mem: Option<ReplMemory>,
) -> IoResult<()> {
    let mut state = ReplState::new(mem);
    let (tx_llm, rx_llm) = std::sync::mpsc::channel::<LlmUiMsg>();

    state.push_thinking("Cantrik REPL — /help for commands, Ctrl+C to quit.".to_string());
    if state.mem.is_some() {
        state.push_thinking("Session memory: ON (SQLite).".to_string());
    } else {
        state.push_thinking(
            "Session memory: OFF (could not open DB — check ~/.local/share/cantrik/).".to_string(),
        );
    }

    loop {
        terminal.draw(|frame| ui(frame, &state, &config))?;

        while let Ok(msg) = rx_llm.try_recv() {
            match msg {
                LlmUiMsg::Chunk(s) => {
                    state.output.push_str(&s);
                }
                LlmUiMsg::Done(Ok(()), assistant_text) => {
                    state.busy = false;
                    state.output.push('\n');
                    state.push_thinking("assistant: stream finished OK.".to_string());
                    if let Some(ref m) = state.mem
                        && let Err(e) = rt.block_on(append_message(
                            &m.pool,
                            &m.session_id,
                            "assistant",
                            &assistant_text,
                        ))
                    {
                        state.push_thinking(format!("session save: {e}"));
                    }
                }
                LlmUiMsg::Done(Err(e), _) => {
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
                    "/cost · /memory · /plan · /agents · /visualize [callgraph|architecture|dependencies] · /doctor · /exit",
                );
            }
            SlashCmd::Visualize(tail) => {
                let kind = visualize_kind_from_tail(tail.as_deref());
                match visualize::render_visualize_kind(kind, cwd) {
                    Ok(text) => {
                        state.preview = text.clone();
                        state.output.push_str("\n--- visualize ---\n");
                        state.output.push_str(&text);
                        state.output.push('\n');
                        state.push_thinking(format!(
                            "visualize: {kind:?} (enable [ui] tui_split_pane for side preview)."
                        ));
                    }
                    Err(e) => state.push_thinking(format!("visualize: {e}")),
                }
            }
            SlashCmd::Cost => {
                let fp = session_project_fingerprint(cwd, config);
                let ym = current_year_month_utc();
                let fut = async {
                    let pool = if let Some(m) = &state.mem {
                        m.pool.clone()
                    } else {
                        connect_pool()
                            .await
                            .map_err(|e: cantrik_core::session::SessionError| e.to_string())?
                    };
                    let month = month_spend_usd(&pool, &fp, &ym)
                        .await
                        .map_err(|e| e.to_string())?;
                    if let Some(m) = &state.mem {
                        let s = session_spend_usd(&pool, &fp, &m.session_id)
                            .await
                            .map_err(|e| e.to_string())?;
                        Ok::<_, String>((Some(s), month))
                    } else {
                        Ok((None, month))
                    }
                };
                match rt.block_on(fut) {
                    Ok((Some(s), month)) => {
                        state.push_thinking(format!("approx session spend: {s:.6} USD"));
                        state.push_thinking(format!(
                            "approx month ({ym}, UTC, this project): {month:.6} USD"
                        ));
                    }
                    Ok((None, month)) => {
                        state.push_thinking(
                            "no active REPL session — open one by sending a message first."
                                .to_string(),
                        );
                        state.push_thinking(format!(
                            "approx month ({ym}, UTC, this project): {month:.6} USD"
                        ));
                    }
                    Err(e) => state.push_thinking(format!("/cost: {e}")),
                }
            }
            SlashCmd::Memory => {
                state.push_thinking(format!("Tier2 DB: {}", memory_db_path().display()));
                let mem = state
                    .mem
                    .as_ref()
                    .map(|m| (m.pool.clone(), m.session_id.clone()));
                if let Some((pool, sid)) = mem {
                    state.push_thinking(format!("  session_id: {}", sid));
                    match rt.block_on(message_count(&pool, &sid)) {
                        Ok(n) => state.push_thinking(format!("  messages: {n}")),
                        Err(e) => state.push_thinking(format!("  count error: {e}")),
                    }
                } else {
                    state.push_thinking("  session: (not connected)".to_string());
                }
                let anc = load_anchors_combined(cwd);
                state.push_thinking(format!("  anchors loaded: {} chars", anc.len()));
                state.push_thinking(
                    "  Tier3: cantrik index + search; Tier4: adaptive_stub in DB (skeleton)."
                        .to_string(),
                );
            }
            SlashCmd::Doctor => {
                state.push_thinking("—— doctor ——".to_string());
                for l in doctor::report_lines(cwd) {
                    state.push_thinking(l);
                }
            }
            SlashCmd::Agents(goal_opt) => {
                let Some(goal_s) = goal_opt.as_deref() else {
                    state.push_thinking(
                        "agents: usage /agents <goal> (multi-agent orchestrator)".to_string(),
                    );
                    return Ok(false);
                };
                state.busy = true;
                state.push_thinking(format!(
                    "agents: orchestrator… ({})",
                    &goal_s[..goal_s.len().min(60)]
                ));
                let cfg = config.clone();
                let cwd_buf = cwd.to_path_buf();
                let code = rt.block_on(agents_cmd::run(
                    &cfg,
                    cwd_buf.as_path(),
                    goal_s,
                    false,
                    None,
                ));
                state.busy = false;
                if code == ExitCode::SUCCESS {
                    state.push_thinking("agents: done.".to_string());
                } else {
                    state.push_thinking("agents: error (see stderr).".to_string());
                }
            }
            SlashCmd::Plan(task_opt) => {
                let status_only = task_opt.is_none();
                let task_s = task_opt.as_deref().unwrap_or("");
                state.busy = true;
                state.push_thinking(if status_only {
                    "plan: loading saved state…".to_string()
                } else {
                    format!(
                        "plan: generating structured plan… ({})",
                        &task_s[..task_s.len().min(60)]
                    )
                });
                let cfg = config.clone();
                let cwd_buf = cwd.to_path_buf();
                let code = rt.block_on(plan_cmd::run(
                    &cfg,
                    cwd_buf.as_path(),
                    task_s,
                    false,
                    status_only,
                ));
                state.busy = false;
                if code == ExitCode::SUCCESS {
                    state.push_thinking("plan: done.".to_string());
                } else {
                    state.push_thinking(
                        "plan: exited with error (see terminal if stderr).".to_string(),
                    );
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

    let mem = state
        .mem
        .as_ref()
        .map(|m| (m.pool.clone(), m.session_id.clone()));
    let full_prompt = if let Some((ref pool, ref session_id)) = mem {
        if let Err(e) = rt.block_on(maybe_summarize_session(pool, session_id, cwd, config)) {
            state.push_thinking(format!("summarize warn: {e}"));
        }
        if let Err(e) = rt.block_on(append_message(pool, session_id, "user", &prompt)) {
            state.push_thinking(format!("session: {e}"));
            state.busy = false;
            return Ok(false);
        }
        match rt.block_on(build_llm_prompt(pool, session_id, cwd, config, &prompt)) {
            Ok(p) => p,
            Err(e) => {
                state.push_thinking(format!("context build: {e}"));
                state.busy = false;
                return Ok(false);
            }
        }
    } else {
        let p = prompt.clone();
        match cultural_wisdom::prompt_addon(effective_cultural_wisdom(&config.ui)) {
            Some(cw) => format!("{cw}\n\nuser: {p}\n"),
            None => p,
        }
    };

    let cfg = config.clone();
    let tx_chunk = tx_llm.clone();
    let tx_done = tx_llm.clone();
    let acc = Arc::new(Mutex::new(String::new()));
    let acc2 = acc.clone();
    let fp = session_project_fingerprint(cwd, &cfg);
    rt.spawn(async move {
        let r = if let Some((ref pool, ref session_id)) = mem {
            let usage = llm::LlmUsageContext {
                pool,
                session_id: Some(session_id.as_str()),
                project_fingerprint: &fp,
            };
            llm::ask_stream_chunks_with(&cfg, &full_prompt, Some(&prompt), Some(usage), &mut |s| {
                let _ = tx_chunk.send(LlmUiMsg::Chunk(s.to_string()));
                acc2.lock().expect("acc").push_str(s);
                Ok(())
            })
            .await
        } else {
            llm::ask_stream_chunks_with(&cfg, &full_prompt, Some(&prompt), None, &mut |s| {
                let _ = tx_chunk.send(LlmUiMsg::Chunk(s.to_string()));
                acc2.lock().expect("acc").push_str(s);
                Ok(())
            })
            .await
        };
        let body = acc.lock().expect("acc").clone();
        let _ = tx_done.send(LlmUiMsg::Done(r.map_err(|e: LlmError| e.to_string()), body));
    });

    Ok(false)
}

fn visualize_kind_from_tail(tail: Option<&str>) -> VisualizeKind {
    match tail
        .map(str::trim)
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "architecture" | "arch" => VisualizeKind::Architecture,
        "dependencies" | "deps" => VisualizeKind::Dependencies,
        "" | "callgraph" | "graph" => VisualizeKind::Callgraph,
        _ => VisualizeKind::Callgraph,
    }
}

fn parse_slash(line: &str) -> Option<SlashCmd> {
    let t = line.trim();
    if !t.starts_with('/') {
        return None;
    }
    let rest = t.strip_prefix('/')?.trim_start();
    let end_head = rest.find(char::is_whitespace).unwrap_or(rest.len());
    let head = rest[..end_head].to_ascii_lowercase();
    let tail = rest[end_head..].trim();
    if head.is_empty() {
        return Some(SlashCmd::Help);
    }
    Some(match head.as_str() {
        "cost" => SlashCmd::Cost,
        "memory" => SlashCmd::Memory,
        "doctor" => SlashCmd::Doctor,
        "plan" => SlashCmd::Plan(if tail.is_empty() {
            None
        } else {
            Some(tail.to_string())
        }),
        "agents" => SlashCmd::Agents(if tail.is_empty() {
            None
        } else {
            Some(tail.to_string())
        }),
        "visualize" | "viz" => SlashCmd::Visualize(if tail.is_empty() {
            None
        } else {
            Some(tail.to_string())
        }),
        "help" | "?" => SlashCmd::Help,
        "exit" | "quit" => SlashCmd::Exit,
        other => SlashCmd::Unknown(other.to_string()),
    })
}

fn ui(frame: &mut ratatui::Frame, state: &ReplState, config: &AppConfig) {
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

    if effective_tui_split_pane(&config.ui) {
        let h = Layout::horizontal([Constraint::Percentage(58), Constraint::Percentage(42)]);
        let [assistant_area, preview_area] = h.areas(main_area);
        let vis_a = assistant_area.height.saturating_sub(2) as usize;
        let main_text = tail_lines(&state.output, vis_a.max(1));
        let main_block = Block::default()
            .borders(Borders::ALL)
            .title(" assistant (streaming) ")
            .border_style(Style::default().fg(Color::Cyan));
        let main_para = Paragraph::new(main_text).block(main_block);
        frame.render_widget(main_para, assistant_area);
        let vis_p = preview_area.height.saturating_sub(2) as usize;
        let prev_text = tail_lines(&state.preview, vis_p.max(1));
        let prev_block = Block::default()
            .borders(Borders::ALL)
            .title(" preview / visualize ")
            .border_style(Style::default().fg(Color::Magenta));
        let prev_para = Paragraph::new(prev_text).block(prev_block);
        frame.render_widget(prev_para, preview_area);
    } else {
        let visible = main_area.height.saturating_sub(2) as usize;
        let main_text = tail_lines(&state.output, visible.max(1));
        let main_block = Block::default()
            .borders(Borders::ALL)
            .title(" assistant (streaming) ")
            .border_style(Style::default().fg(Color::Cyan));
        let main_para = Paragraph::new(main_text).block(main_block);
        frame.render_widget(main_para, main_area);
    }

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
            parse_slash("/visualize architecture"),
            Some(SlashCmd::Visualize(Some(s))) if s == "architecture"
        ));
        assert!(matches!(
            parse_slash("/nope"),
            Some(SlashCmd::Unknown(x)) if x == "nope"
        ));
        assert!(parse_slash("hello").is_none());
        assert!(matches!(parse_slash("/plan"), Some(SlashCmd::Plan(None))));
        assert!(matches!(
            parse_slash("/plan fix thing"),
            Some(SlashCmd::Plan(Some(s))) if s == "fix thing"
        ));
        assert!(matches!(
            parse_slash("/agents do x"),
            Some(SlashCmd::Agents(Some(s))) if s == "do x"
        ));
    }
}
