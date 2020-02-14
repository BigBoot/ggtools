use iced::{
    button,
    settings::Window,
    text_input,
    Align,
    Application,
    Button,
    Checkbox,
    Color,
    Column,
    Command,
    Container,
    Element,
    Length,
    Row,
    Settings,
    Text,
    TextInput,
};
use nfd::Response;
use qbsdiff::Bspatch;
use rgcp_common::config::Config;

#[derive(Debug, Clone)]
pub struct PatchError(pub String);

impl<T: std::fmt::Display> From<T> for PatchError {
    #[inline]
    fn from(d: T) -> Self {
        PatchError(d.to_string())
    }
}

pub type PatchResult = ::std::result::Result<(), PatchError>;

#[derive(Default)]
struct Counter {
    target_path: String,
    patches: Vec<(Patch, bool)>,
    status_msg: String,
    status_color: [f32; 4],
    file_select_input: text_input::State,
    browse_button: button::State,
    patch_button: button::State,
}

#[derive(Debug, Clone)]
enum Message {
    BrowsePressed,
    FileSelectInputChanged(String),
    PatchSelected(String, bool),
    PatchPressed,
}

#[derive(Debug, Clone)]
struct Patch {
    id: String,
    description: String,
    diff: Vec<u8>,
}

impl Patch {
    fn new(id: &str, description: &str, diff: &[u8]) -> Self {
        Self { id: id.to_owned(), description: description.to_owned(), diff: diff.to_vec() }
    }
}

impl Counter {
    fn set_status(&mut self, msg: &str, color: Color) {
        self.status_msg = msg.to_owned();
        self.status_color = [color.r, color.g, color.b, color.a];
    }

    fn apply_patches(&mut self) -> PatchResult {
        let patches = self
            .patches
            .iter()
            .filter_map(|(patch, is_checked)| {
                if *is_checked {
                    Some(patch.clone())
                }
                else {
                    None
                }
            })
            .collect::<Vec<Patch>>();


        let target = std::path::PathBuf::from(self.target_path.clone());

        self.set_status(&format!("Loading file {}", &self.target_path), Color::BLACK);
        let mut data = std::fs::read(target.clone())?;

        for patch in &patches {
            self.set_status(&format!("Applying patch {}", &patch.description), Color::BLACK);

            let patcher = Bspatch::new(&patch.diff)?.buffer_size(4096);
            let mut target = Vec::new();

            patcher.apply(&data, std::io::Cursor::new(&mut target))?;

            data = target;
        }

        let out_file = std::path::PathBuf::from(target.clone())
            .with_file_name(
                target.file_stem().and_then(|s| s.to_str()).map(|s| format!("{}-Patched", s)).unwrap_or(format!("")),
            )
            .with_extension(target.extension().and_then(|s| s.to_str()).unwrap_or(""));

        self.set_status(&format!("Writing file {}", &out_file.display()), Color::BLACK);
        std::fs::write(out_file, data)?;

        self.set_status(&format!("Successfully applied {} patches", patches.len()), Color::BLACK);

        Ok(())
    }
}

impl Application for Counter {
    type Message = Message;

    fn new() -> (Self, Command<Message>) {
        (
            Self {
                patches: vec![(Patch::new("0gb", "Disable RAM check", include_bytes!("../patches/0gb.patch")), true)],
                ..Self::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Gigantic Patcher")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::BrowsePressed => {
                let result = nfd::dialog().default_path(&self.target_path).filter("exe").open().unwrap_or_else(|e| {
                    panic!(e);
                });
                match result {
                    Response::Okay(file) => self.target_path = file,
                    Response::OkayMultiple(files) => {
                        files.iter().take(1).for_each(|file| self.target_path = file.clone())
                    },
                    Response::Cancel => {},
                }
            },
            Message::FileSelectInputChanged(value) => {
                self.target_path = value;
            },
            Message::PatchSelected(id, selected) => {
                for (patch, is_checked) in &mut self.patches {
                    if *patch.id == id {
                        *is_checked = selected;
                    }
                }
            },
            Message::PatchPressed => {
                if let Err(e) = self.apply_patches() {
                    self.set_status(&format!("{}", e.0), [1f32, 0f32, 0f32, 1f32].into());
                }
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let mut patches_container = Column::new();

        for (patch, is_checked) in &self.patches {
            let id = patch.id.clone();
            patches_container =
                patches_container.push(Checkbox::new(*is_checked, &patch.description, move |selected| {
                    Message::PatchSelected(id.clone(), selected)
                }));
        }

        Container::new(
            Column::new()
                .push(
                    Row::new()
                        .push(TextInput::new(
                            &mut self.file_select_input,
                            "RxGame-Win64-Test.exe",
                            &self.target_path,
                            Message::FileSelectInputChanged,
                        ))
                        .push(
                            Button::new(&mut self.browse_button, Text::new("\u{2026}"))
                                .on_press(Message::BrowsePressed),
                        )
                        .spacing(20),
                )
                .push(patches_container)
                .push(Button::new(&mut self.patch_button, Text::new("Patch")).on_press(Message::PatchPressed))
                .push(Text::new(self.status_msg.clone()).color(self.status_color))
                .align_items(Align::Center)
                .spacing(20)
                .padding(20),
        )
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}

pub async fn run(_config: Config) {
    Counter::run(Settings {
        window: Window { size: (300, 200), resizable: false, ..Window::default() },
        ..Settings::default()
    })
}
