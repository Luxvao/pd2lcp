#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs::File, path::PathBuf, sync::LazyLock};

use color_eyre::eyre::Result;
use iced::{
    Alignment, Border, Color, Element,
    Length::Fill,
    Subscription, Task, Theme,
    futures::{SinkExt, Stream},
    stream,
    widget::{
        Space, button, checkbox, column as col, container, pick_list, progress_bar, row, text,
    },
};

#[cfg(target_os = "windows")]
use libpd2lcp::error::Error;

#[cfg(target_os = "windows")]
use std::path::Path;

use libpd2lcp::{
    base_game::{install_d2, install_d2_lod},
    event::{Event, EventNotify},
    launch::launch,
    pd2_updater::{install_pd2, update_available},
    settings::{GraphicsMode, Settings},
    state::State,
};

static EVENT_NOTIFY: LazyLock<EventNotify> = LazyLock::new(EventNotify::default);

mod palette {
    use iced::Color;

    pub const BACKGROUND: Color = Color::from_rgb(0.102, 0.102, 0.118); // #1a1a1e
    pub const SURFACE: Color = Color::from_rgb(0.141, 0.141, 0.161); // #242429
    pub const BORDER: Color = Color::from_rgb(0.220, 0.220, 0.247); // #38383f
    pub const TEXT: Color = Color::from_rgb(0.949, 0.949, 0.953); // #f2f2f3
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.604, 0.604, 0.635); // #9a9aa2
    pub const ACCENT: Color = Color::from_rgb(0.357, 0.557, 0.937); // #5b8def
    pub const ERROR: Color = Color::from_rgb(0.898, 0.282, 0.298); // #e5484d
}

mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
}

fn lighten(c: Color, amount: f32) -> Color {
    Color {
        r: (c.r + amount).min(1.0),
        g: (c.g + amount).min(1.0),
        b: (c.b + amount).min(1.0),
        a: c.a,
    }
}

fn darken(c: Color, amount: f32) -> Color {
    Color {
        r: (c.r - amount).max(0.0),
        g: (c.g - amount).max(0.0),
        b: (c.b - amount).max(0.0),
        a: c.a,
    }
}

fn page_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::BACKGROUND.into()),
        text_color: Some(palette::TEXT),
        ..Default::default()
    }
}

fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::SURFACE.into()),
        border: Border {
            color: palette::BORDER,
            width: 1.0,
            radius: 12.0.into(),
        },
        text_color: Some(palette::TEXT),
        ..Default::default()
    }
}

fn error_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(palette::SURFACE.into()),
        border: Border {
            color: palette::ERROR,
            width: 1.0,
            radius: 12.0.into(),
        },
        text_color: Some(palette::TEXT),
        ..Default::default()
    }
}

fn primary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let base = palette::ACCENT;
    let background = match status {
        button::Status::Hovered => lighten(base, 0.06),
        button::Status::Pressed => darken(base, 0.08),
        button::Status::Disabled => Color { a: 0.35, ..base },
        button::Status::Active => base,
    };

    button::Style {
        background: Some(background.into()),
        text_color: Color::WHITE,
        border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn secondary_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let border_color = match status {
        button::Status::Hovered => palette::ACCENT,
        _ => palette::BORDER,
    };

    button::Style {
        background: Some(Color::TRANSPARENT.into()),
        text_color: palette::TEXT,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..Default::default()
    }
}

fn progress_style(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: palette::BACKGROUND.into(),
        bar: palette::ACCENT.into(),
        border: Border {
            color: palette::BORDER,
            width: 1.0,
            radius: 6.0.into(),
        },
    }
}

fn settings_section<'a>(
    title: &'a str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    col![
        text(title.to_uppercase())
            .size(12)
            .color(palette::TEXT_SECONDARY),
        Space::new().height(space::SM),
        content.into(),
    ]
    .spacing(space::XS)
    .into()
}

fn page<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(
        container(content)
            .padding(space::XL)
            .max_width(420.0)
            .style(card_style),
    )
    .width(Fill)
    .height(Fill)
    .center(Fill)
    .style(page_style)
    .into()
}

fn step_indicator<'a>(current: u8, total: u8) -> Element<'a, Message> {
    text(format!("STEP {current} OF {total}"))
        .size(12)
        .color(palette::ACCENT)
        .into()
}

#[derive(Debug)]
struct Launcher {
    game_state: Option<State>,
    gui_state: GuiState,
    settings: Settings,
    launch_button_label: &'static str,
    init_screen_text: &'static str,
    error_prev_screen: Option<GuiState>,
    installing_progress: Option<(u32, u32)>,
    launch_button_disabled: bool,
    allow_next: bool,
    disable_get_started: bool,
}

#[derive(Clone, Debug)]
enum GuiState {
    Init,
    InstallD2,
    InstallLOD,
    Main,
    Error(String),
    Settings,
}

#[derive(Clone, Debug)]
enum Message {
    InstallerExit(Result<(), String>),
    LaunchTask(Result<(), String>),
    NotifyEvent(Event),
    InitGameState(Result<State, String>),
    InitButton,
    UpdateCheckTask(Result<bool, String>),
    UpdateTask(Result<(), String>),
    LaunchButton,
    SettingsButton,
    GraphicsMode(GraphicsMode),
    SoundCheckbox(bool),
    BnetCheckbox(bool),
    UpdatesCheckbox(bool),
    D2InstallerSelected(Option<PathBuf>),
    LODInstallerSelected(Option<PathBuf>),
    ApplyButton,
    ReturnButton,
    NextButton,
    FilePickerButtonD2,
    FilePickerButtonLOD,
    ErrorReturnButton,
}

fn update(state: &mut Launcher, message: Message) -> Task<Message> {
    match message {
        Message::InstallerExit(res) => {
            let _ = handle_fallible(state, res, |state, _| {
                state.allow_next = true;
                None
            });
        }
        Message::InitButton => {
            state.disable_get_started = true;

            return Task::perform(State::init(EVENT_NOTIFY.clone()), |game_state| {
                Message::InitGameState(game_state.map_err(|e| e.to_string()))
            });
        }
        Message::NextButton => {
            state.allow_next = false;
            match state.gui_state {
                GuiState::InstallD2 => state.gui_state = GuiState::InstallLOD,
                GuiState::InstallLOD => {
                    state.gui_state = GuiState::Main;

                    if let Err(e) = File::create(
                        state
                            .game_state
                            .as_ref()
                            .expect("pd2lcp is not initialised")
                            .base()
                            .join("setup_finished"),
                    ) {
                        state.gui_state = GuiState::Error(e.to_string());
                    };
                }
                _ => (),
            }
        }
        Message::FilePickerButtonD2 => {
            return Task::perform(
                rfd::AsyncFileDialog::new()
                    .add_filter("exe", &["exe"])
                    .set_can_create_directories(false)
                    .set_title("Select D2 Installer")
                    .pick_file(),
                |p| Message::D2InstallerSelected(p.map(|p| p.path().to_path_buf())),
            );
        }
        Message::FilePickerButtonLOD => {
            return Task::perform(
                rfd::AsyncFileDialog::new()
                    .add_filter("exe", &["exe"])
                    .set_can_create_directories(false)
                    .set_title("Select D2 LOD Installer")
                    .pick_file(),
                |p| Message::LODInstallerSelected(p.map(|p| p.path().to_path_buf())),
            );
        }
        Message::D2InstallerSelected(p) => {
            if let Some(path) = p {
                return Task::perform(
                    install_d2(
                        state.game_state.clone().expect("pd2lcp is not initialised"),
                        path,
                    ),
                    |r| Message::InstallerExit(r.map_err(|r| r.to_string())),
                );
            }
        }
        Message::LODInstallerSelected(p) => {
            if let Some(path) = p {
                return Task::perform(
                    install_d2_lod(
                        state.game_state.clone().expect("pd2lcp is not initialised"),
                        path,
                    ),
                    |r| Message::InstallerExit(r.map_err(|r| r.to_string())),
                );
            }
        }
        Message::InitGameState(res) => {
            let _ = handle_fallible(state, res, |state, game_state| {
                state.game_state = Some(game_state);
                state.gui_state = GuiState::InstallD2;

                None
            });
        }
        Message::SettingsButton => state.gui_state = GuiState::Settings,
        Message::ReturnButton => state.gui_state = GuiState::Main,
        Message::GraphicsMode(new) => state.settings.graphics_mode = new,
        Message::SoundCheckbox(b) => state.settings.sndbkg = b,
        Message::BnetCheckbox(b) => state.settings.skiptobnet = b,
        Message::UpdatesCheckbox(b) => state.settings.no_updates = b,
        Message::ApplyButton => {
            let game_state = state
                .game_state
                .as_ref()
                .expect("pd2lcp is not initialised");

            state.gui_state = GuiState::Main;

            let _ = handle_fallible(
                state,
                game_state
                    .serialise_settings(&state.settings)
                    .map_err(|e| e.to_string()),
                |_, _| None,
            );
        }
        Message::ErrorReturnButton => {
            state.gui_state = state.error_prev_screen.take().expect("this cannot happen")
        }
        Message::LaunchButton => {
            state.launch_button_disabled = true;

            if state.settings.no_updates {
                return Task::perform(
                    launch(
                        state.game_state.clone().expect("pd2lcp is not initialised"),
                        state.settings.clone(),
                    ),
                    |r| Message::LaunchTask(r.map_err(|e| e.to_string())),
                );
            } else {
                return Task::perform(
                    update_available(state.game_state.clone().expect("pd2lcp is not initialised")),
                    |r| Message::UpdateCheckTask(r.map_err(|e| e.to_string())),
                );
            }
        }
        Message::UpdateCheckTask(res) => {
            if let Some(task) = handle_fallible(state, res, |state, b| {
                if b {
                    state.launch_button_label = "Updating";

                    return Some(Task::perform(
                        install_pd2(
                            state.game_state.clone().expect("pd2lcp is not initialised"),
                            EVENT_NOTIFY.clone(),
                        ),
                        |r| Message::UpdateTask(r.map_err(|e| e.to_string())),
                    ));
                }

                Some(Task::perform(
                    launch(
                        state.game_state.clone().expect("pd2lcp is not initialised"),
                        state.settings.clone(),
                    ),
                    |r| Message::LaunchTask(r.map_err(|e| e.to_string())),
                ))
            }) {
                return task;
            }
        }
        Message::UpdateTask(res) => {
            if let Some(task) = handle_fallible(state, res, |state, _| {
                state.launch_button_label = "Launch";
                state.installing_progress = None;

                Some(Task::perform(
                    launch(
                        state.game_state.clone().expect("pd2lcp is not initialised"),
                        state.settings.clone(),
                    ),
                    |r| Message::LaunchTask(r.map_err(|e| e.to_string())),
                ))
            }) {
                return task;
            }
        }
        Message::LaunchTask(res) => {
            let _ = handle_fallible(state, res, |state, _| {
                state.launch_button_disabled = false;
                None
            });
        }
        Message::NotifyEvent(event) => match event {
            Event::UpdatingPD2 { done, total } => {
                state.installing_progress = Some((done, total));
            }
            Event::DoneUpdating => {
                state.installing_progress = None;
            }
            Event::Error(e) => {
                state.error_prev_screen = Some(state.gui_state.clone());
                state.gui_state = GuiState::Error(e);
            }
            Event::DownloadingWine => state.init_screen_text = "Downloading Wine...",
            Event::FinishedDownloadingWine => (),
            Event::InitPrefix => state.init_screen_text = "Setting up the prefix...",
            Event::FinishedInitPrefix => state.init_screen_text = "Done!",
        },
    }

    Task::none()
}

fn view(state: &Launcher) -> Element<'_, Message> {
    match state.gui_state {
        GuiState::Init => init_screen(state),
        GuiState::InstallD2 => install_d2_screen(state),
        GuiState::InstallLOD => install_d2_lod_screen(state),
        GuiState::Main => main_screen(state),
        GuiState::Settings => settings_screen(state),
        GuiState::Error(ref e) => error_screen(e),
    }
}

fn init_screen(state: &Launcher) -> Element<'_, Message> {
    let content = col![
        text("Welcome").size(22).color(palette::TEXT),
        text(state.init_screen_text)
            .size(14)
            .color(palette::TEXT_SECONDARY),
        Space::new().height(space::MD),
        if state.disable_get_started {
            button(text("Get Started").center())
                .width(Fill)
                .padding(space::MD)
                .style(primary_button_style)
        } else {
            button(text("Get Started").center())
                .on_press(Message::InitButton)
                .width(Fill)
                .padding(space::MD)
                .style(primary_button_style)
        }
    ]
    .spacing(space::SM)
    .width(Fill);

    page(content)
}

fn install_d2_screen(state: &Launcher) -> Element<'_, Message> {
    let content = col![
        step_indicator(1, 2),
        text("Install Diablo II").size(22).color(palette::TEXT),
        text("Select your Diablo II installer. Select \"A:\\Diablo II\" as the install path")
            .size(14)
            .color(palette::TEXT_SECONDARY),
        Space::new().height(space::LG),
        button(text("Select D2 Installer").center())
            .on_press(Message::FilePickerButtonD2)
            .width(Fill)
            .padding(space::MD)
            .style(secondary_button_style),
        Space::new().height(space::SM),
        if state.allow_next {
            button(text("Next").center())
                .on_press(Message::NextButton)
                .width(Fill)
                .padding(space::MD)
                .style(primary_button_style)
        } else {
            button(text("Next").center())
                .width(Fill)
                .padding(space::MD)
                .style(primary_button_style)
        }
    ]
    .spacing(space::SM)
    .width(Fill);

    page(content)
}

fn install_d2_lod_screen(state: &Launcher) -> Element<'_, Message> {
    let content = col![
        step_indicator(2, 2),
        text("Install Lord of Destruction").size(22).color(palette::TEXT),
        text(
            "Select your D2 LOD installer. It will install into the same location as the base game by default."
        )
        .size(14)
        .color(palette::TEXT_SECONDARY),
        Space::new().height(space::LG),
        button(text("Select D2 LOD Installer").center())
            .on_press(Message::FilePickerButtonLOD)
            .width(Fill)
            .padding(space::MD)
            .style(secondary_button_style),
        Space::new().height(space::SM),
        if state.allow_next {
            button(text("Next").center())
                .on_press(Message::NextButton)
                .width(Fill)
                .padding(space::MD)
                .style(primary_button_style)
        } else {
            button(text("Next").center())
                .width(Fill)
                .padding(space::MD)
                .style(primary_button_style)
        }
    ]
    .spacing(space::SM)
    .width(Fill);

    page(content)
}

fn main_screen(state: &Launcher) -> Element<'_, Message> {
    let header = row![
        text("Project Diablo II")
            .size(20)
            .color(palette::TEXT)
            .width(Fill),
        button(text("Settings").size(13))
            .on_press(Message::SettingsButton)
            .padding([space::XS, space::SM])
            .style(secondary_button_style),
    ]
    .align_y(iced::Alignment::Center);

    let mut content = col![header, Space::new().height(space::LG)]
        .spacing(space::SM)
        .width(Fill);

    content = content.push(if state.launch_button_disabled {
        button(text(state.launch_button_label).center())
            .width(Fill)
            .padding(space::MD)
            .style(primary_button_style)
    } else {
        button(text(state.launch_button_label).center())
            .on_press(Message::LaunchButton)
            .width(Fill)
            .padding(space::MD)
            .style(primary_button_style)
    });

    if let Some((done, total)) = state.installing_progress {
        content = content.push(Space::new().height(space::MD));

        content = content.push(
            col![
                text(format!("Downloading - {done}/{total}"))
                    .size(13)
                    .color(palette::TEXT_SECONDARY),
                progress_bar(0f32..=total as f32, done as f32).style(progress_style),
            ]
            .spacing(space::XS)
            .width(Fill),
        );
    }

    page(content)
}

fn settings_screen(state: &Launcher) -> Element<'_, Message> {
    let renderers = [GraphicsMode::DDRAW, GraphicsMode::_3DFX];

    let content = col![
        row![
            text("Settings").size(20).color(palette::TEXT).width(Fill),
            button(text("Back").size(13))
                .on_press(Message::ReturnButton)
                .padding([space::XS, space::SM])
                .style(secondary_button_style),
        ]
        .align_y(iced::Alignment::Center),
        Space::new().height(space::XS),
        settings_section(
            "Graphics",
            row![
                text("Renderer"),
                Space::new().width(space::SM),
                pick_list(
                    renderers,
                    Some(state.settings.graphics_mode),
                    Message::GraphicsMode
                )
                .style(|_, _| pick_list::Style {
                    text_color: Color::WHITE,
                    placeholder_color: Color::WHITE,
                    background: palette::SURFACE.into(),
                    handle_color: Color::WHITE,
                    border: Border {
                        radius: 8.0.into(),
                        color: palette::BORDER,
                        width: 1.0,
                    },
                })
            ]
            .align_y(Alignment::Center)
        ),
        Space::new().height(space::MD),
        settings_section(
            "Sound",
            checkbox(state.settings.sndbkg)
                .label("Sound in background")
                .on_toggle(Message::SoundCheckbox)
        ),
        Space::new().height(space::LG),
        settings_section(
            "Misc",
            col![
                checkbox(state.settings.skiptobnet)
                    .label("Skip to battlenet")
                    .on_toggle(Message::BnetCheckbox),
                Space::new().height(space::XS),
                checkbox(state.settings.no_updates)
                    .label("Disable updates")
                    .on_toggle(Message::UpdatesCheckbox)
            ]
        ),
        container(
            button(text("Apply").size(13))
                .style(primary_button_style)
                .on_press(Message::ApplyButton)
        )
        .align_right(Fill)
    ]
    .spacing(space::SM)
    .width(Fill);

    page(content)
}

fn error_screen(message: &str) -> Element<'_, Message> {
    let content = col![
        text("Something went wrong").size(20).color(palette::ERROR),
        text(message).size(14).color(palette::TEXT_SECONDARY),
        Space::new().height(space::LG),
        button(text("Return").center())
            .on_press(Message::ErrorReturnButton)
            .width(Fill)
            .padding(space::MD)
            .style(primary_button_style),
    ]
    .spacing(space::SM)
    .width(Fill);

    container(
        container(content)
            .padding(space::XL)
            .max_width(420.0)
            .style(error_card_style),
    )
    .width(Fill)
    .height(Fill)
    .center(Fill)
    .style(page_style)
    .into()
}

fn worker() -> impl Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let notify = EVENT_NOTIFY.clone();

        loop {
            for event in notify.wait_event().await.expect("mutex poisoned, exiting") {
                output
                    .send(Message::NotifyEvent(event))
                    .await
                    .expect("event notify stream broken");
            }
        }
    })
}

fn subscription(_: &Launcher) -> Subscription<Message> {
    Subscription::run(worker)
}

fn main() -> Result<()> {
    #[cfg(not(feature = "gamemode"))]
    let fullscreen = std::env::args().any(|arg| arg == "-gamemode");

    #[cfg(feature = "gamemode")]
    let fullscreen = true;

    iced::application(
        || {
            let state_raw = State::init_raw().expect("Failed to init initial state");
            let settings = state_raw.deserialise_settings();

            let base_path = state_raw.base();

            let setup_finished_path = base_path.join("setup_finished");

            let (game_state, gui_state) = if !setup_finished_path.exists() {
                if base_path.exists() {
                    #[cfg(target_os = "linux")]
                    std::fs::remove_dir_all(base_path).expect("Failed to clean up ~/Games/pd2lcp");

                    #[cfg(target_os = "windows")]
                    cleanup_base_win(base_path).expect("Failed to clean up A:\\");
                }

                #[cfg(target_os = "linux")]
                let ret = (None, GuiState::Init);

                #[cfg(target_os = "windows")]
                let ret = (
                    Some(State::init_raw().expect("Failed to init initial state")),
                    GuiState::InstallD2,
                );

                ret
            } else {
                (Some(state_raw), GuiState::Main)
            };

            Launcher {
                game_state,
                gui_state,
                settings,
                error_prev_screen: None,
                launch_button_label: "Launch",
                init_screen_text: "PD2LCP is not set up",
                installing_progress: None,
                launch_button_disabled: false,
                allow_next: false,
                disable_get_started: false,
            }
        },
        update,
        view,
    )
    .window(iced::window::Settings {
        fullscreen,
        ..Default::default()
    })
    .subscription(subscription)
    .run()?;

    Ok(())
}

fn handle_fallible<T, F>(
    state: &mut Launcher,
    res: Result<T, String>,
    f: F,
) -> Option<Task<Message>>
where
    F: FnOnce(&mut Launcher, T) -> Option<Task<Message>>,
{
    match res {
        Ok(o) => return f(state, o),
        Err(e) => {
            state.error_prev_screen = Some(state.gui_state.clone());
            state.gui_state = GuiState::Error(e);
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn cleanup_base_win(path: &Path) -> Result<(), Error> {
    use std::fs::read_dir;

    for entry in read_dir(path)? {
        if let Ok(entry) = entry {
            let entry = entry.path();

            // Skip the launcher itself
            if let Some(name) = entry.file_name() {
                if name == "pd2lcp-iced.exe" {
                    continue;
                }
            }

            if entry.is_dir() {
                std::fs::remove_dir_all(entry)?;
            } else {
                std::fs::remove_file(entry)?;
            }
        }
    }

    Ok(())
}
