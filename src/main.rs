use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use crossbeam_channel::{Receiver, Sender};
use log::{error, warn};
use nix::errno::Errno;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use winit::event::{Event, StartCause};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};

const POLL_INTERVAL: Duration = Duration::from_secs(2);
const SIGTERM_GRACE: Duration = Duration::from_secs(2);
const SIGKILL_GRACE: Duration = Duration::from_secs(1);
const POLL_STEP: Duration = Duration::from_millis(200);
const MAX_TOOLTIP_ENTRIES: usize = 5;
const MENU_ID_KILL_ALL: &str = "kill_all";
const MENU_ID_QUIT: &str = "quit";
const MENU_ID_EDIT_CONFIG: &str = "edit_config";
const MENU_ID_PROCESS_PREFIX: &str = "process_";
const MENU_ID_EMPTY: &str = "empty";

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Config {
    port_ranges: Vec<(u16, u16)>,
    #[serde(default = "default_inactive_color")]
    inactive_color: (u8, u8, u8),
    #[serde(default = "default_active_color")]
    active_color: (u8, u8, u8),
}

fn default_inactive_color() -> (u8, u8, u8) {
    (255, 255, 255) // White - matches other macOS menu bar icons
}

fn default_active_color() -> (u8, u8, u8) {
    (255, 69, 58) // Red - SF Symbols system red color
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port_ranges: vec![
                (3000, 3010),   // Node.js, React, Next.js, Vite
                (3306, 3306),   // MySQL
                (4000, 4010),   // Alternative Node servers
                (5001, 5010),   // Flask, general dev servers (excluding 5000)
                (5173, 5173),   // Vite default
                (5432, 5432),   // PostgreSQL
                (6379, 6379),   // Redis
                (8000, 8100),   // Django, Python HTTP servers
                (8080, 8090),   // Tomcat, alternative HTTP
                (9000, 9010),   // Various dev tools
                (27017, 27017), // MongoDB
            ],
            inactive_color: default_inactive_color(),
            active_color: default_active_color(),
        }
    }
}

fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".macport.json")
}

fn load_or_create_config() -> Result<Config> {
    let path = get_config_path();

    if path.exists() {
        let content = fs::read_to_string(&path)
            .context("failed to read config file")?;
        serde_json::from_str(&content)
            .context("failed to parse config file")
    } else {
        let config = Config::default();
        save_config(&config)?;
        Ok(config)
    }
}

fn save_config(config: &Config) -> Result<()> {
    let path = get_config_path();
    let content = serde_json::to_string_pretty(config)
        .context("failed to serialize config")?;
    fs::write(&path, content)
        .context("failed to write config file")?;
    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();

    let config = load_or_create_config().context("failed to load configuration")?;

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .context("failed to create event loop")?;
    let proxy = event_loop.create_proxy();
    let (kill_tx, kill_rx) = crossbeam_channel::unbounded();

    let _monitor_thread = spawn_monitor_thread(proxy.clone(), config.clone());
    let _menu_thread = spawn_menu_listener(proxy.clone());
    let _kill_worker = spawn_kill_worker(kill_rx, proxy.clone());

    let icon = create_icon(config.inactive_color).context("failed to create tray icon image")?;
    let initial_menu = build_menu(&[]).context("failed to build initial menu")?;
    let tray_icon = TrayIconBuilder::new()
        .with_icon(icon)
        .with_menu(Box::new(initial_menu))
        .with_tooltip("No dev port listeners detected.")
        .build()
        .context("failed to create tray icon")?;
    tray_icon
        .set_visible(true)
        .context("failed to show tray icon")?;

    let mut state = AppState {
        processes: Vec::new(),
        last_feedback: None,
        config: config.clone(),
    };
    update_tray_display(&tray_icon, &state);
    let mut kill_sender: Option<Sender<KillCommand>> = Some(kill_tx);

    #[allow(deprecated)]
    let run_result = event_loop.run(move |event, event_loop| match event {
        Event::NewEvents(StartCause::Init) => {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
        Event::UserEvent(user_event) => match user_event {
            UserEvent::ProcessesUpdated(processes) => {
                state.processes = processes;
                sync_menu(&tray_icon, &state.processes);
                update_tray_display(&tray_icon, &state);
            }
            UserEvent::MenuAction(action) => match action {
                MenuAction::EditConfig => {
                    let config_path = get_config_path();
                    let path_str = config_path.to_string_lossy().to_string();
                    let _ = Command::new("open")
                        .arg("-t")
                        .arg(&path_str)
                        .spawn();
                    state.last_feedback = Some(KillFeedback::info(format!(
                        "Opened config file: {}",
                        path_str
                    )));
                    update_tray_display(&tray_icon, &state);
                }
                MenuAction::KillPid { pid, .. } => {
                    if let Some(target) = describe_pid(pid, &state.processes) {
                        if let Some(sender) = kill_sender.as_ref() {
                            if let Err(err) = sender.send(KillCommand::KillPid(target)) {
                                let feedback = KillFeedback::error(format!(
                                    "Unable to dispatch kill command: {}",
                                    err
                                ));
                                kill_sender = None;
                                state.last_feedback = Some(feedback);
                                update_tray_display(&tray_icon, &state);
                            }
                        } else {
                            let feedback = KillFeedback::error(format!(
                                "Kill worker unavailable for PID {}.",
                                pid
                            ));
                            state.last_feedback = Some(feedback);
                            update_tray_display(&tray_icon, &state);
                        }
                    } else {
                        state.last_feedback = Some(KillFeedback::info(format!(
                            "PID {} is no longer active.",
                            pid
                        )));
                        update_tray_display(&tray_icon, &state);
                    }
                }
                MenuAction::KillAll => {
                    let targets = collect_targets_for_all(&state.processes);
                    if targets.is_empty() {
                        state.last_feedback = Some(KillFeedback::info(
                            "No dev port listeners to terminate.".to_string(),
                        ));
                        update_tray_display(&tray_icon, &state);
                    } else if let Some(sender) = kill_sender.as_ref() {
                        if let Err(err) = sender.send(KillCommand::KillAll(targets)) {
                            let feedback = KillFeedback::error(format!(
                                "Unable to dispatch kill-all command: {}",
                                err
                            ));
                            kill_sender = None;
                            state.last_feedback = Some(feedback);
                            update_tray_display(&tray_icon, &state);
                        }
                    } else {
                        let feedback = KillFeedback::error(
                            "Kill worker unavailable for batch request.".to_string(),
                        );
                        state.last_feedback = Some(feedback);
                        update_tray_display(&tray_icon, &state);
                    }
                }
                MenuAction::Quit => {
                    event_loop.exit();
                }
            },
            UserEvent::KillFeedback(feedback) => {
                state.last_feedback = Some(feedback);
                update_tray_display(&tray_icon, &state);
            }
            UserEvent::MonitorError(message) => {
                warn!("Monitor error: {}", message);
                state.last_feedback = Some(KillFeedback::error(message));
                update_tray_display(&tray_icon, &state);
            }
        },
        Event::LoopExiting => {
            kill_sender.take();
        }
        _ => {}
    });

    run_result.context("event loop terminated with error")?;
    Ok(())
}

fn spawn_monitor_thread(proxy: EventLoopProxy<UserEvent>, config: Config) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut previous: Vec<ProcessInfo> = Vec::new();
        loop {
            match scan_ports(&config.port_ranges) {
                Ok(mut processes) => {
                    processes.sort();
                    if processes != previous {
                        previous = processes.clone();
                        if proxy
                            .send_event(UserEvent::ProcessesUpdated(processes))
                            .is_err()
                        {
                            break;
                        }
                    }
                }
                Err(err) => {
                    let message = format!("{}", err);
                    if proxy.send_event(UserEvent::MonitorError(message)).is_err() {
                        break;
                    }
                }
            }
            thread::sleep(POLL_INTERVAL);
        }
    })
}

fn spawn_menu_listener(proxy: EventLoopProxy<UserEvent>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let receiver = MenuEvent::receiver().clone();
        for event in receiver.iter() {
            let Some(action) = parse_menu_action(event.id()) else {
                continue;
            };
            if proxy.send_event(UserEvent::MenuAction(action)).is_err() {
                break;
            }
        }
    })
}

fn spawn_kill_worker(
    rx: Receiver<KillCommand>,
    proxy: EventLoopProxy<UserEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for command in rx.iter() {
            let should_continue = match command {
                KillCommand::KillPid(target) => handle_single_kill(&proxy, target),
                KillCommand::KillAll(targets) => handle_batch_kill(&proxy, targets),
            };
            if !should_continue {
                break;
            }
        }
    })
}

fn handle_single_kill(proxy: &EventLoopProxy<UserEvent>, target: KillTarget) -> bool {
    let outcome = terminate_pid(target.pid);
    let feedback = match outcome {
        KillOutcome::Success => {
            KillFeedback::info(format!("Terminated {} (PID {}).", target.label, target.pid))
        }
        KillOutcome::AlreadyExited => KillFeedback::warning(format!(
            "{} (PID {}) was already stopped.",
            target.label, target.pid
        )),
        KillOutcome::PermissionDenied => KillFeedback::error(format!(
            "Permission denied terminating {} (PID {}).",
            target.label, target.pid
        )),
        KillOutcome::TimedOut => KillFeedback::error(format!(
            "Timed out terminating {} (PID {}).",
            target.label, target.pid
        )),
        KillOutcome::Failed(err) => KillFeedback::error(format!(
            "Failed to terminate {} (PID {}): {}.",
            target.label, target.pid, err
        )),
    };

    proxy.send_event(UserEvent::KillFeedback(feedback)).is_ok()
}

fn handle_batch_kill(proxy: &EventLoopProxy<UserEvent>, targets: Vec<KillTarget>) -> bool {
    if targets.is_empty() {
        return proxy
            .send_event(UserEvent::KillFeedback(KillFeedback::info(
                "No dev port listeners to terminate.".to_string(),
            )))
            .is_ok();
    }

    let mut successes = 0usize;
    let mut already = 0usize;
    let mut denied = 0usize;
    let mut timed_out = 0usize;
    let mut failures: Vec<(KillTarget, Errno)> = Vec::new();

    for target in targets {
        match terminate_pid(target.pid) {
            KillOutcome::Success => successes += 1,
            KillOutcome::AlreadyExited => already += 1,
            KillOutcome::PermissionDenied => {
                denied += 1;
                failures.push((target, Errno::EPERM));
            }
            KillOutcome::TimedOut => {
                timed_out += 1;
                failures.push((target, Errno::ETIMEDOUT));
            }
            KillOutcome::Failed(err) => failures.push((target, err)),
        }
    }

    let failure_count = failures.len();
    let severity = if successes > 0 && failure_count == 0 && denied == 0 && timed_out == 0 {
        FeedbackSeverity::Info
    } else if successes > 0 {
        FeedbackSeverity::Warning
    } else {
        FeedbackSeverity::Error
    };

    let mut parts = Vec::new();
    if successes > 0 {
        parts.push(format!("terminated {}", successes));
    }
    if already > 0 {
        parts.push(format!("{} already stopped", already));
    }
    if denied > 0 {
        parts.push(format!("{} permission denied", denied));
    }
    if timed_out > 0 {
        parts.push(format!("{} timed out", timed_out));
    }
    if failure_count > 0 {
        parts.push(format!("{} failed", failure_count));
    }

    if parts.is_empty() {
        parts.push("no action taken".to_string());
    }

    let mut message = format!("Kill all: {}.", parts.join(", "));
    if let Some((failed_target, err)) = failures.first() {
        message.push_str(&format!(
            " First failure: {} (PID {}) — {}.",
            failed_target.label, failed_target.pid, err
        ));
    }

    let feedback = KillFeedback::new(message, severity);
    proxy.send_event(UserEvent::KillFeedback(feedback)).is_ok()
}

fn terminate_pid(pid_raw: i32) -> KillOutcome {
    let pid = Pid::from_raw(pid_raw);

    match kill(pid, None) {
        Err(Errno::ESRCH) => return KillOutcome::AlreadyExited,
        Err(err) => return KillOutcome::Failed(err),
        Ok(()) => {}
    }

    match kill(pid, Signal::SIGTERM) {
        Ok(()) => {}
        Err(Errno::ESRCH) => return KillOutcome::AlreadyExited,
        Err(Errno::EPERM) => return KillOutcome::PermissionDenied,
        Err(err) => return KillOutcome::Failed(err),
    }

    match wait_for_exit(pid, SIGTERM_GRACE) {
        Ok(true) => return KillOutcome::Success,
        Ok(false) => {}
        Err(Errno::EPERM) => return KillOutcome::PermissionDenied,
        Err(err) => return KillOutcome::Failed(err),
    }

    match kill(pid, Signal::SIGKILL) {
        Ok(()) => {}
        Err(Errno::ESRCH) => return KillOutcome::Success,
        Err(Errno::EPERM) => return KillOutcome::PermissionDenied,
        Err(err) => return KillOutcome::Failed(err),
    }

    match wait_for_exit(pid, SIGKILL_GRACE) {
        Ok(true) => KillOutcome::Success,
        Ok(false) => KillOutcome::TimedOut,
        Err(Errno::EPERM) => KillOutcome::PermissionDenied,
        Err(err) => KillOutcome::Failed(err),
    }
}

fn wait_for_exit(pid: Pid, timeout: Duration) -> Result<bool, Errno> {
    let deadline = Instant::now() + timeout;
    loop {
        match kill(pid, None) {
            Err(Errno::ESRCH) => return Ok(true),
            Err(err) => return Err(err),
            Ok(()) => {}
        }

        if Instant::now() >= deadline {
            return Ok(false);
        }
        thread::sleep(POLL_STEP);
    }
}

fn scan_ports(port_ranges: &[(u16, u16)]) -> Result<Vec<ProcessInfo>> {
    let mut results = Vec::new();
    let mut seen = HashSet::new();
    let mut command_cache: HashMap<i32, String> = HashMap::new();

    for &(start, end) in port_ranges {
        for port in start..=end {
            let pids = query_port(port)?;
            for pid in pids {
                if !seen.insert((port, pid)) {
                    continue;
                }
                let command = command_cache
                    .entry(pid)
                    .or_insert_with(|| resolve_command(pid));
                results.push(ProcessInfo {
                    port,
                    pid,
                    command: command.clone(),
                });
            }
        }
    }

    results.sort();
    Ok(results)
}

fn query_port(port: u16) -> Result<Vec<i32>> {
    let output = Command::new("lsof")
        .args(["-ti", &format!(":{}", port), "-sTCP:LISTEN"])
        .output()
        .with_context(|| format!("failed to execute lsof for port {}", port))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut pids = Vec::new();
        for line in stdout.lines() {
            if let Ok(pid) = line.trim().parse::<i32>() {
                pids.push(pid);
            }
        }
        Ok(pids)
    } else if output.status.code() == Some(1) && output.stdout.is_empty() {
        Ok(Vec::new())
    } else {
        Err(anyhow!(
            "lsof reported error for port {}: {}",
            port,
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn resolve_command(pid: i32) -> String {
    let output = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let command = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if command.is_empty() {
                format!("pid {}", pid)
            } else {
                command
            }
        }
        Ok(output) => {
            warn!(
                "ps failed for pid {}: {}",
                pid,
                String::from_utf8_lossy(&output.stderr)
            );
            format!("pid {}", pid)
        }
        Err(err) => {
            warn!("ps execution error for pid {}: {}", pid, err);
            format!("pid {}", pid)
        }
    }
}

fn parse_menu_action(id: &MenuId) -> Option<MenuAction> {
    let raw = id.as_ref();
    if raw == MENU_ID_KILL_ALL {
        Some(MenuAction::KillAll)
    } else if raw == MENU_ID_QUIT {
        Some(MenuAction::Quit)
    } else if raw == MENU_ID_EDIT_CONFIG {
        Some(MenuAction::EditConfig)
    } else if let Some(remainder) = raw.strip_prefix(MENU_ID_PROCESS_PREFIX) {
        let mut parts = remainder.split('_');
        let pid = parts.next()?.parse::<i32>().ok()?;
        let _port = parts.next()?.parse::<u16>().ok()?;
        Some(MenuAction::KillPid { pid })
    } else {
        None
    }
}

fn collect_targets_for_all(processes: &[ProcessInfo]) -> Vec<KillTarget> {
    let mut map: BTreeMap<i32, (String, Vec<u16>)> = BTreeMap::new();

    for process in processes {
        let entry = map
            .entry(process.pid)
            .or_insert_with(|| (process.command.clone(), Vec::new()));
        if !entry.1.contains(&process.port) {
            entry.1.push(process.port);
        }
        if entry.0.is_empty() {
            entry.0 = process.command.clone();
        }
    }

    map.into_iter()
        .filter_map(|(pid, (command, mut ports))| {
            if ports.is_empty() {
                return None;
            }
            ports.sort();
            let label = format_command_label(&command, &ports);
            Some(KillTarget { pid, label })
        })
        .collect()
}

fn describe_pid(pid: i32, processes: &[ProcessInfo]) -> Option<KillTarget> {
    let mut ports = Vec::new();
    let mut command: Option<String> = None;
    for process in processes.iter().filter(|p| p.pid == pid) {
        if !ports.contains(&process.port) {
            ports.push(process.port);
        }
        if command.is_none() || command.as_deref().unwrap().starts_with("pid ") {
            command = Some(process.command.clone());
        }
    }

    if ports.is_empty() {
        return None;
    }

    ports.sort();
    let label = format_command_label(command.as_deref().unwrap_or(""), &ports);
    Some(KillTarget { pid, label })
}

fn format_command_label(command: &str, ports: &[u16]) -> String {
    let mut label = if command.is_empty() {
        "Unknown".to_string()
    } else {
        command.to_string()
    };
    if !ports.is_empty() {
        label.push_str(" (port");
        if ports.len() > 1 {
            label.push('s');
        }
        label.push(' ');
        for (idx, port) in ports.iter().enumerate() {
            if idx > 0 {
                label.push_str(", ");
            }
            label.push_str(&port.to_string());
        }
        label.push(')');
    }
    label
}

fn build_menu(processes: &[ProcessInfo]) -> Result<Menu> {
    let menu = Menu::new();
    if processes.is_empty() {
        let item = MenuItem::with_id(MENU_ID_EMPTY, "No dev ports listening", false, None);
        menu.append(&item)?;
    } else {
        for process in processes {
            let label = format!(
                "Kill {} (PID {}, port {})",
                process.command, process.pid, process.port
            );
            let item = MenuItem::with_id(
                MenuId::new(process_menu_id(process.pid, process.port)),
                label,
                true,
                None,
            );
            menu.append(&item)?;
        }
        menu.append(&PredefinedMenuItem::separator())?;
        let kill_all_label = format!("Kill all ({})", processes.len());
        let kill_all_item = MenuItem::with_id(MENU_ID_KILL_ALL, kill_all_label, true, None);
        menu.append(&kill_all_item)?;
    }

    menu.append(&PredefinedMenuItem::separator())?;
    let edit_config_item = MenuItem::with_id(MENU_ID_EDIT_CONFIG, "Edit Configuration...", true, None);
    menu.append(&edit_config_item)?;
    let quit_item = MenuItem::with_id(MENU_ID_QUIT, "Quit", true, None);
    menu.append(&quit_item)?;
    Ok(menu)
}

fn process_menu_id(pid: i32, port: u16) -> MenuId {
    MenuId::new(format!("{}{}_{}", MENU_ID_PROCESS_PREFIX, pid, port))
}

fn sync_menu(tray_icon: &TrayIcon, processes: &[ProcessInfo]) {
    match build_menu(processes) {
        Ok(menu) => tray_icon.set_menu(Some(Box::new(menu))),
        Err(err) => error!("Failed to rebuild menu: {}", err),
    }
}

fn update_tray_display(
    tray_icon: &TrayIcon,
    state: &AppState,
) {
    // Update icon color based on whether ports are active
    let color = if state.processes.is_empty() {
        state.config.inactive_color
    } else {
        state.config.active_color
    };

    if let Ok(icon) = create_icon(color) {
        let _ = tray_icon.set_icon(Some(icon));
    }

    let tooltip = build_tooltip(&state.processes, state.last_feedback.as_ref());
    if let Err(err) = tray_icon.set_tooltip(Some(tooltip.as_str())) {
        error!("Failed to update tooltip: {}", err);
    }
}

fn build_tooltip(processes: &[ProcessInfo], feedback: Option<&KillFeedback>) -> String {
    let mut lines = Vec::new();
    if processes.is_empty() {
        lines.push("No dev port listeners detected.".to_string());
    } else {
        lines.push(format!("Active listeners: {}", processes.len()));
        for process in processes.iter().take(MAX_TOOLTIP_ENTRIES) {
            lines.push(format!(
                "Port {} → {} (PID {})",
                process.port, process.command, process.pid
            ));
        }
        if processes.len() > MAX_TOOLTIP_ENTRIES {
            lines.push(format!(
                "…and {} more",
                processes.len() - MAX_TOOLTIP_ENTRIES
            ));
        }
    }

    if let Some(feedback) = feedback {
        let prefix = match feedback.severity {
            FeedbackSeverity::Info => "",
            FeedbackSeverity::Warning => "⚠️ ",
            FeedbackSeverity::Error => "⛔ ",
        };
        lines.push(format!("Last action: {}{}", prefix, feedback.message));
    }

    lines.join("\n")
}

fn create_icon(color: (u8, u8, u8)) -> Result<Icon> {
    // Create a 22x22 icon (standard macOS menu bar size)
    let size = 22;
    let mut pixels = vec![0u8; (size * size * 4) as usize];

    let (r, g, b) = color;

    let draw_pixel = |pixels: &mut [u8], x: i32, y: i32, alpha: u8| {
        if x >= 0 && x < size && y >= 0 && y < size {
            let idx = ((y * size + x) * 4) as usize;
            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = alpha;
        }
    };

    // Draw a circular port icon with connection lines
    // Outer circle (port)
    for angle in 0..360 {
        let rad = (angle as f32).to_radians();
        let x = 11 + (7.0 * rad.cos()) as i32;
        let y = 11 + (7.0 * rad.sin()) as i32;
        draw_pixel(&mut pixels, x, y, 255);
    }

    // Inner circle fill (lighter)
    for dy in -5..=5 {
        for dx in -5..=5 {
            if dx * dx + dy * dy <= 25 {
                draw_pixel(&mut pixels, 11 + dx, 11 + dy, 180);
            }
        }
    }

    // Center dot (darker)
    for dy in -2..=2 {
        for dx in -2..=2 {
            if dx * dx + dy * dy <= 4 {
                draw_pixel(&mut pixels, 11 + dx, 11 + dy, 255);
            }
        }
    }

    // Connection lines (top and bottom)
    for i in 0..3 {
        draw_pixel(&mut pixels, 11, 3 + i, 255); // Top line
        draw_pixel(&mut pixels, 11, 16 + i, 255); // Bottom line
    }

    Icon::from_rgba(pixels, size as u32, size as u32)
        .map_err(|err| anyhow!("failed to build icon: {err}"))
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct ProcessInfo {
    port: u16,
    pid: i32,
    command: String,
}

#[derive(Clone, Debug)]
enum UserEvent {
    ProcessesUpdated(Vec<ProcessInfo>),
    MenuAction(MenuAction),
    KillFeedback(KillFeedback),
    MonitorError(String),
}

#[derive(Clone, Debug)]
enum MenuAction {
    KillPid { pid: i32 },
    KillAll,
    EditConfig,
    Quit,
}

#[derive(Clone, Debug)]
enum KillCommand {
    KillPid(KillTarget),
    KillAll(Vec<KillTarget>),
}

#[derive(Clone, Debug)]
struct KillTarget {
    pid: i32,
    label: String,
}

#[derive(Clone, Debug)]
struct KillFeedback {
    message: String,
    severity: FeedbackSeverity,
}

impl KillFeedback {
    fn new(message: String, severity: FeedbackSeverity) -> Self {
        Self { message, severity }
    }

    fn info(message: String) -> Self {
        Self::new(message, FeedbackSeverity::Info)
    }

    fn warning(message: String) -> Self {
        Self::new(message, FeedbackSeverity::Warning)
    }

    fn error(message: String) -> Self {
        Self::new(message, FeedbackSeverity::Error)
    }
}

#[derive(Clone, Copy, Debug)]
enum FeedbackSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
struct AppState {
    processes: Vec<ProcessInfo>,
    last_feedback: Option<KillFeedback>,
    config: Config,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            processes: Vec::new(),
            last_feedback: None,
            config: Config::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum KillOutcome {
    Success,
    AlreadyExited,
    PermissionDenied,
    TimedOut,
    Failed(Errno),
}
