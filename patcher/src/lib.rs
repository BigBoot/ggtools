use chrono::Utc;
use iced::{Align, Application, Button, Clipboard, Column, Command, Container, Element, Image, Length, PickList, Row, Settings, Space, Text, TextInput, button, executor, pick_list, text_input, window::Settings as Window, image::Handle};
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
struct Patcher {
    target_path: String,
    patches: Vec<Patch>,
    file_select_input: text_input::State,
    browse_button: button::State,
    patch_button: button::State,
    patch_selector: pick_list::State<Patch>,
    selected_patch: Patch,
}

#[derive(Debug, Clone)]
enum Message {
    BrowsePressed,
    FileSelectInputChanged(String),
    PatchSelected(Patch),
    PatchPressed,
}

#[derive(Debug, Clone, Eq, Default)]
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

impl std::fmt::Display for Patch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::cmp::PartialEq for Patch {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Patcher {
    fn set_status(&mut self, msg: &str) {
        eprintln!("{}", &msg);
    }

    fn apply_patch(&mut self) -> PatchResult {
        
        let target = std::path::PathBuf::from(self.target_path.clone());

        if !target.exists() {
            return Err(PatchError("The selected file does not exists.".to_owned()));
        }

        self.set_status(&format!("Loading file {}", &self.target_path));
        let data = std::fs::read(target.clone())?;

        self.set_status(&format!("Applying patch {}", &self.selected_patch.description));

        let patcher = Bspatch::new(&self.selected_patch.diff)?.buffer_size(4096);
        let mut result = Vec::new();

        patcher.apply(&data, std::io::Cursor::new(&mut result))?;

        let backup = std::path::PathBuf::from(target.clone())
            .with_file_name(
                target.file_stem().and_then(|s| s.to_str()).map(|s| format!("{}_{}", s, Utc::now().format("%Y%m%d_%H%M%S"))).unwrap_or(format!("")),
            )
            .with_extension(target.extension().and_then(|s| s.to_str()).unwrap_or(""));

        self.set_status("Creating backup");
        std::fs::rename(&target, &backup)?;

        self.set_status(&format!("Writing file {}", &target.display()));
        std::fs::write(target, data)?;

        self.set_status(&format!("Successfully applied patch"));

        msgbox::create(
            "Patch successfull", 
            &format!("The selected patch has been applied succesfully!"), 
            msgbox::IconType::None,
        )?;

        Ok(())
    }
}

impl Application for Patcher {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let patches = vec![
            Patch::new("0gb", "Disable RAM check", include_bytes!("../patches/0gb.patch")),
            Patch::new("remote_set", "Enable set command", include_bytes!("../patches/remote_set.patch")),
        ];

        (
            Self {
                selected_patch: patches.first().map(ToOwned::to_owned).expect("No patches available!"),
                patches: patches,
                ..Self::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Gigantic Patcher")
    }

    fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::BrowsePressed => {
                let result = nfd::dialog().default_path(&self.target_path).filter("exe").open().unwrap_or_else(|e| {
                    panic!("{}", e);
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
            Message::PatchSelected(patch) => {
                self.selected_patch = patch;
            },
            Message::PatchPressed => {
                if let Err(e) = self.apply_patch() {
                    msgbox::create(
                        "Error patching the executable", 
                        &format!("An error occured while trying to patch the executable, please make sure you've selected the correct file!\n\n{}", &e.0), 
                        msgbox::IconType::Error,
                    ).expect("Error showing error message, what a pity...");
                    self.set_status(&format!("{}", e.0));
                }
            },
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        let file_selector_input = TextInput::new(
            &mut self.file_select_input,
            "",
            &self.target_path,
            Message::FileSelectInputChanged,
        );

        let file_selector_button = Button::new(
            &mut self.browse_button, 
            Text::new("Browse")
        )
        .on_press(Message::BrowsePressed);

        let patch_selector = PickList::new(&mut self.patch_selector, 
            &self.patches, 
            Some(self.selected_patch.clone()), 
            Message::PatchSelected
        );

        let patch_button = Button::new(
            &mut self.patch_button, Text::new("Patch")
        )
        .on_press(Message::PatchPressed);

        Container::new(
            Column::new()
                .push(Text::new("File to patch:").width(Length::Fill))
                .push(Row::new()
                    .push(file_selector_input.padding(10).size(20))
                    .push(file_selector_button.padding(10))
                )
                .push(Space::new(Length::Fill, 5.into()))
                .push(patch_selector.width(Length::Fill))
                .push(Space::new(Length::Fill, 5.into()))
                .push(patch_button.padding(10))
                .align_items(Align::Center)
                .spacing(5)
                .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into()
    }
}

pub fn run(_config: Config) {
    if cfg!(windows) {
        use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS}; 
        unsafe { AttachConsole(ATTACH_PARENT_PROCESS); }
    }

    Patcher::run(Settings {
        window: Window { size: (300, 220), resizable: false, ..Window::default() },
        ..Settings::default()
    }).unwrap();
}
