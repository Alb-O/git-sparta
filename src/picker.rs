use anyhow::{Result, anyhow};
use devicons::FileIcon;
use nucleo_picker::{PickerOptions, Render, error::PickError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttributeRow {
    pub name: String,
    pub count: usize,
}

impl AttributeRow {
    pub fn new(name: impl Into<String>, count: usize) -> Self {
        Self {
            name: name.into(),
            count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileRow {
    pub path: String,
    pub tags: Vec<String>,
}

impl FileRow {
    pub fn new(path: impl Into<String>, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            path: path.into(),
            tags: tags.into_iter().map(|tag| tag.into()).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchSelection {
    Attribute(AttributeRow),
    File(FileRow),
}

#[derive(Clone, Debug, Default)]
pub struct UiConfig;

impl UiConfig {
    pub fn tags_and_files() -> Self {
        Self
    }
}

#[derive(Clone, Debug, Default)]
pub struct SearchData {
    context: Option<String>,
    initial_query: Option<String>,
    attributes: Vec<AttributeRow>,
    files: Vec<FileRow>,
}

impl SearchData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_initial_query(mut self, query: impl Into<String>) -> Self {
        self.initial_query = Some(query.into());
        self
    }

    pub fn with_attributes(mut self, attributes: Vec<AttributeRow>) -> Self {
        self.attributes = attributes;
        self
    }

    pub fn with_files(mut self, files: Vec<FileRow>) -> Self {
        self.files = files;
        self
    }
}

pub struct SearchUi {
    data: SearchData,
    ui_config: UiConfig,
    input_title: Option<String>,
}

impl SearchUi {
    pub fn new(data: SearchData) -> Self {
        Self {
            data,
            ui_config: UiConfig,
            input_title: None,
        }
    }

    pub fn with_ui_config(mut self, ui_config: UiConfig) -> Self {
        self.ui_config = ui_config;
        self
    }

    pub fn with_input_title(mut self, title: impl Into<String>) -> Self {
        self.input_title = Some(title.into());
        self
    }

    pub fn with_theme_name(self, _name: &str) -> Self {
        // Theme selection is not currently supported by the nucleo picker integration.
        self
    }

    pub fn run(self) -> Result<SearchOutcome> {
        let mut options = PickerOptions::new();
        if let Some(query) = &self.data.initial_query {
            options = options.query(query.clone());
        }

        let mut picker = options.picker(EntryRenderer);

        let injector = nucleo_picker::Picker::injector(&picker);
        for entry in build_entries(self.data, &self.ui_config, self.input_title.as_deref()) {
            injector.push(entry);
        }

        let pick_result = nucleo_picker::Picker::pick(&mut picker);

        let outcome = match pick_result {
            Ok(opt) => {
                let selection = opt.map(|entry| entry.selection.clone());
                let query = nucleo_picker::Picker::query(&picker).to_string();
                SearchOutcome {
                    accepted: selection.is_some(),
                    query,
                    selection,
                }
            }
            Err(PickError::UserInterrupted) => {
                let query = nucleo_picker::Picker::query(&picker).to_string();
                SearchOutcome {
                    accepted: false,
                    query,
                    selection: None,
                }
            }
            Err(PickError::NotInteractive) => {
                return Err(anyhow!(
                    "interactive picker requires an interactive stderr; rerun in a terminal or pass --yes"
                ));
            }
            Err(PickError::Disconnected) => {
                return Err(anyhow!("picker event channel disconnected"));
            }
            Err(PickError::IO(err)) => return Err(err.into()),
            Err(_) => unreachable!("application never provides abort errors to the picker"),
        };

        Ok(outcome)
    }
}

pub struct SearchOutcome {
    pub accepted: bool,
    pub query: String,
    pub selection: Option<SearchSelection>,
}

#[derive(Clone, Debug)]
struct PickerEntry {
    render: String,
    selection: SearchSelection,
}

const ATTRIBUTE_ICON: char = '󰊢';
const GENERIC_FILE_ICON: &str = "󰈔";

fn build_entries(data: SearchData, _config: &UiConfig, _title: Option<&str>) -> Vec<PickerEntry> {
    fn assert_send_sync_static<T: Send + Sync + 'static>() {}
    assert_send_sync_static::<PickerEntry>();

    let mut entries = Vec::new();

    if !data.attributes.is_empty() {
        for attribute in data.attributes.into_iter() {
            let render = format!(
                "{ATTRIBUTE_ICON} {name}  ({count} matches)",
                name = attribute.name,
                count = attribute.count
            );
            entries.push(PickerEntry {
                render,
                selection: SearchSelection::Attribute(attribute),
            });
        }
    }

    for file in data.files.into_iter() {
        let icon = FileIcon::from(file.path.as_str());
        let icon_string = icon.to_string();
        let icon = if icon_string == "*" {
            GENERIC_FILE_ICON
        } else {
            icon_string.as_str()
        };
        let mut render = format!("{icon} {}", file.path);
        if !file.tags.is_empty() {
            render.push_str("  [");
            render.push_str(&file.tags.join(", "));
            render.push(']');
        }
        entries.push(PickerEntry {
            render,
            selection: SearchSelection::File(file),
        });
    }

    entries
}

struct EntryRenderer;

impl Render<PickerEntry> for EntryRenderer {
    type Str<'a>
        = &'a str
    where
        PickerEntry: 'a;

    fn render<'a>(&self, item: &'a PickerEntry) -> Self::Str<'a> {
        item.render.as_str()
    }
}
