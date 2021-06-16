mod config;

use anyhow::{Context, Result};
use crossterm::event::{self, Event as CEvent, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::collections::VecDeque;
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tui::backend::CrosstermBackend;
use tui::layout::Corner;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem};
use tui::Terminal;
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::{PrivmsgMessage, RGBColor, ServerMessage};
use twitch_irc::{ClientConfig, TCPTransport, TwitchIRCClient};

#[derive(Debug)]
enum Event {
    ChatMessage(ServerMessage),
    Input(CEvent),
    Render,
}

fn luminance(color: &RGBColor) -> f32 {
    0.2126 * (color.r as f32) + 0.7152 * (color.g as f32) + 0.00722 * (color.b as f32)
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let config = config::Config::load()?;
    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();

    // default configuration is to join chat as anonymous.
    let irc_config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<TCPTransport, StaticLoginCredentials>::new(irc_config);

    let tx2 = tx.clone();
    tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            tx2.send(Event::ChatMessage(message))
                .expect("sending chat message event");
        }
    });
    let tick_rate = Duration::from_millis(200);
    tokio::spawn(async move {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(CEvent::Key(key)))
                        .expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Render) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    client.join(config.channel.to_owned());

    enable_raw_mode().context("failed to enable raw mode")?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("can't create terminal backend")?;
    terminal.clear().context("can't clear terminal")?;
    let mut messages: VecDeque<PrivmsgMessage> = VecDeque::new();
    loop {
        match rx.recv().await.expect("receiving event") {
            Event::Input(event) => {
                if let CEvent::Key(key) = event {
                    // CTRL-C -> exit
                    if key.modifiers == crossterm::event::KeyModifiers::CONTROL {
                        if let KeyCode::Char('c') = key.code {
                            disable_raw_mode().expect("disable_raw_mode");
                            terminal.show_cursor().expect("show cursor");
                            std::process::exit(0);
                        }
                    }
                }
            }
            Event::ChatMessage(message) => {
                if let ServerMessage::Privmsg(privmsg) = message {
                    messages.push_front(privmsg);
                    if messages.len() > config.messages_buffer_size {
                        messages.pop_back();
                    }
                }
            }
            Event::Render => {
                terminal
                    .draw(|f| {
                        let size = f.size();
                        let mut items: Vec<ListItem> = vec![];
                        let debug = false;
                        for m in &messages {
                            let style = match &m.name_color {
                                Some(color) => {
                                    let style =
                                        Style::default().fg(Color::Rgb(color.r, color.g, color.b));
                                    if luminance(color) < config.invert_below_brightness as f32 {
                                        style.bg(Color::Gray)
                                    } else {
                                        style
                                    }
                                }
                                None => Style::default(),
                            };

                            let is_subscriber = m.badges.iter().any(|b| b.name == "subscriber");
                            let is_founder = m.badges.iter().any(|b| b.name == "founder");
                            let is_mod = m.badges.iter().any(|b| b.name == "moderator");
                            let is_vip = m.badges.iter().any(|b| b.name == "vip");

                            let mut width_for_name: usize = m.sender.name.len() + 2 /* ": " */;
                            let mut badges = String::new();
                            if is_subscriber {
                                badges.push_str(&config.subscriber_symbol);
                                width_for_name += &config.subscriber_symbol_width;
                            }
                            if is_founder {
                                badges.push_str(&config.founder_symbol);
                                width_for_name += &config.founder_symbol_width;
                            }
                            if is_mod {
                                badges.push_str(&config.mod_symbol);
                                width_for_name += &config.mod_symbol_width;
                            }
                            if is_vip {
                                badges.push_str(&config.vip_symbol);
                                width_for_name += &config.vip_symbol_width;
                            }
                            let width_for_name = width_for_name;
                            let width_for_text: usize = size.width as usize - width_for_name;
                            let lines = textwrap::fill(&m.message_text, width_for_text);
                            let mut lines = lines.split('\n');
                            let mut tmpitems: VecDeque<ListItem> = VecDeque::new();
                            let l = lines.next().expect("message came with no first line");
                            tmpitems.push_front(ListItem::new(Spans(vec![
                                Span::raw(badges),
                                Span::styled(&m.sender.name, style),
                                Span::raw(": "),
                                Span::raw(l.to_owned()),
                            ])));
                            for l in lines {
                                tmpitems.push_front(ListItem::new(Spans(vec![
                                    Span::raw((0..width_for_name).map(|_| " ").collect::<String>()),
                                    Span::raw(l.to_owned()),
                                ])));
                            }
                            for i in tmpitems {
                                items.push(i);
                            }

                            if debug {
                                let i = format!("{:?}", m);
                                let lines = textwrap::fill(&i, size.width as usize);
                                let lines = lines.split('\n');
                                for l in lines.rev() {
                                    items.push(ListItem::new(Spans(vec![Span::raw(l.to_owned())])));
                                }
                            }
                        }

                        let list = List::new(items)
                            .block(Block::default().borders(Borders::NONE))
                            .start_corner(Corner::BottomLeft);
                        f.render_widget(list, size);
                    })
                    .context("unable to draw on terminal")?;
            }
        }
    }
}
