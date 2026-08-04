use std::sync::LazyLock;

use color_eyre::eyre::Result;
use iced::{
    Element,
    Length::Fill,
    Subscription, Task,
    futures::{SinkExt, Stream},
    stream,
    widget::{Space, button, column as col, container, progress_bar, text},
};
use libpd2lcp::{
    event::{Event, EventNotify},
    launch::launch,
    pd2_updater::{install_pd2, update_available},
    settings::Settings,
    setup_test_run,
    state::State,
};

static EVENT_NOTIFY: LazyLock<EventNotify> = LazyLock::new(EventNotify::default);

#[derive(Debug)]
struct Launcher {
    game_state: Option<State>,
    gui_state: GuiState,
    game_settings: Settings,
    launch_button_label: &'static str,
    error_prev_screen: Option<GuiState>,
    installing_progress: Option<(u32, u32)>,
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
    NotifyEvent(Event),
    InitGameState(Result<State, String>),
    InitButton,
    UpdateCheckTask(Result<bool, String>),
    UpdateTask(Result<(), String>),
    Updating,
    LaunchTask(Result<(), String>),
    LaunchButton,
    SettingsButton,
    ApplyButton,
    ReturnButton,
    NextButton,
    FilePickerButton,
    ErrorReturnButton,
}

fn update(state: &mut Launcher, message: Message) -> Task<Message> {
    match message {
        Message::InitButton => {
            return Task::perform(State::init(EVENT_NOTIFY.clone()), |game_state| {
                Message::InitGameState(game_state.map_err(|e| e.to_string()))
            });
        }
        Message::InitGameState(res) => {
            let _ = handle_fallible(state, res, |state, game_state| {
                state.game_state = Some(game_state);
                state.gui_state = GuiState::Main;

                None
            });
        }
        Message::SettingsButton => state.gui_state = GuiState::Settings,
        Message::ReturnButton => state.gui_state = GuiState::Main,
        Message::ErrorReturnButton => {
            state.gui_state = state.error_prev_screen.take().expect("this cannot happen")
        }
        Message::LaunchButton => {
            println!("Launch button pressed");
            return Task::perform(
                update_available(state.game_state.clone().expect("pd2lcp is not initialised")),
                |r| Message::UpdateCheckTask(r.map_err(|e| e.to_string())),
            );
        }
        Message::UpdateCheckTask(res) => {
            println!("update check task hit, state: {:?}", state);

            if let Some(task) = handle_fallible(state, res, |state, b| {
                if b {
                    println!("updating");
                    state.launch_button_label = "Updating";

                    return Some(Task::perform(
                        install_pd2(
                            state.game_state.clone().expect("pd2lcp is not initialised"),
                            EVENT_NOTIFY.clone(),
                        ),
                        |r| Message::UpdateTask(r.map_err(|e| e.to_string())),
                    ));
                }

                println!("launching");
                Some(Task::perform(
                    launch(
                        state.game_state.clone().expect("pd2lcp is not initialised"),
                        state.game_settings.clone(),
                    ),
                    |r| Message::LaunchTask(r.map_err(|e| e.to_string())),
                ))
            }) {
                return task;
            }
        }
        Message::UpdateTask(res) => {
            let _ = handle_fallible(state, res, |state, _| {
                state.installing_progress = None;
                state.launch_button_label = "Launch";

                None
            });
        }
        Message::NotifyEvent(event) => match event {
            Event::UpdatingPD2 { done, total } => {
                state.installing_progress = Some((done, total));
            }
            _ => (),
        },
        _ => (),
    }

    Task::none()
}

fn view(state: &Launcher) -> Element<'_, Message> {
    match state.gui_state {
        GuiState::Init => init_screen(state),
        GuiState::InstallD2 => install_d2_screen(),
        GuiState::InstallLOD => install_d2_lod_screen(),
        GuiState::Main => main_screen(state),
        GuiState::Settings => settings_screen(state),
        GuiState::Error(ref e) => col![
            text(e).width(Fill).height(Fill).center(),
            container(button("Return").on_press(Message::ErrorReturnButton))
                .align_bottom(Fill)
                .align_right(Fill)
        ]
        .into(),
    }
}

fn init_screen(state: &Launcher) -> Element<'_, Message> {
    container(button("Init").on_press(Message::InitButton))
        .center(Fill)
        .into()
}

fn install_d2_screen() -> Element<'static, Message> {
    todo!()
}

fn install_d2_lod_screen() -> Element<'static, Message> {
    todo!()
}

fn main_screen(state: &Launcher) -> Element<'_, Message> {
    col![
        container(button("Settings").on_press(Message::SettingsButton))
            .align_top(Fill)
            .align_right(Fill),
        container(text("Project Diablo II"))
            .align_bottom(Fill)
            .center_x(Fill),
        container(button(state.launch_button_label).on_press(Message::LaunchButton))
            .align_top(Fill)
            .center_x(Fill),
        if let Some((done, total)) = state.installing_progress {
            container(progress_bar(0f32..=total as f32, done as f32))
                .align_top(Fill)
                .center_x(Fill)
        } else {
            container(Space::new())
        }
    ]
    .into()
}

fn settings_screen(state: &Launcher) -> Element<'_, Message> {
    col![
        container(button("Return").on_press(Message::ReturnButton))
            .align_top(Fill)
            .align_right(Fill),
    ]
    .into()
}

fn worker() -> impl Stream<Item = Message> {
    stream::channel(100, async |mut output| {
        let notify = EVENT_NOTIFY.clone();

        for event in notify.wait_event().expect("mutex poisoned, exiting") {
            println!("{:?}", event);

            output
                .send(Message::NotifyEvent(event))
                .await
                .expect("event notify stream broken");
        }
    })
}

fn subscription(_: &Launcher) -> Subscription<Message> {
    Subscription::run(worker)
}

fn main() -> Result<()> {
    // Setup
    setup_test_run();

    println!("Done");

    iced::application(
        || Launcher {
            game_state: None,
            gui_state: GuiState::Init,
            game_settings: Settings {
                graphics: false,
                skiptobnet: true,
                sndbkg: false,
            },
            error_prev_screen: None,
            launch_button_label: "Launch",
            installing_progress: None,
        },
        update,
        view,
    )
    .resizable(false)
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
