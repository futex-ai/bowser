### Core Types

```rust
/// Configuration for launching a browser instance.
pub struct BrowserConfig {
    pub chrome_path: Option<PathBuf>,
    pub chrome_args: Vec<String>,
    pub session: SessionConfig,
    pub output: OutputConfig,
    pub headless: bool,
    pub persistent_profile: bool,
    pub user_data_dir: Option<PathBuf>,
    pub viewport: Viewport,
    pub timeout: Duration,
    pub stealth: bool,
}

/// Configuration for detached Bowser sessions.
pub struct SessionConfig {
    pub id: Option<String>,
    pub dir: Option<PathBuf>,
    pub idle_ttl: Duration,
}

/// Output preview configuration.
pub struct OutputConfig {
    pub truncate: bool,
    pub include_hidden: bool, // legacy/no-op; obscured content is included by default
    pub max_children: usize,
    pub max_list_items: usize,
    pub max_table_rows: usize,
}

/// Viewport dimensions.
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

/// A live browser instance.
pub struct Browser { /* ... */ }

/// Metadata about the attached Bowser session.
pub struct SessionInfo {
    pub id: String,
    pub resumed: bool,
    pub selected_page_id: Option<String>,
}

pub enum SessionPageType {
    Tab,
}

/// Summary of a known page inside a detached Bowser session.
pub struct SessionPageSummary {
    pub id: String,
    pub page_type: SessionPageType,
    pub url: Option<String>,
    pub title: Option<String>,
    pub selected: bool,
    pub live: bool,
}

/// Result of a browser-native file download.
pub struct DownloadResult {
    pub path: PathBuf,
    pub filename: String,
    pub bytes: u64,
}

/// A live browser page with navigation and interaction methods.
pub struct Page { /* ... */ }

/// The structured representation of a rendered page.
pub struct PageCapture {
    pub url: String,
    pub title: String,
    pub body_id: Option<u32>,
    pub obscured_body_id: Option<u32>,
    pub content: PageContent,
}

pub struct PageContent {
    pub visible: Vec<Element>,
    pub obscured: Vec<Element>,
}

/// A single element in the page tree.
pub enum Element {
    Heading {
        level: u8,
        text: String,
    },
    Text {
        text: String,
    },
    Link {
        id: u32,
        text: String,
        href: String,
        focused: bool,
    },
    Button {
        id: u32,
        text: String,
        focused: bool,
    },
    Input {
        id: Option<u32>,
        name: Option<String>,
        input_type: InputType,
        placeholder: Option<String>,
        value: String,
        label: Option<String>,
        options: Vec<String>,  // non-empty for select
        focused: bool,
    },
    Image {
        id: u32,
        alt: String,
        src: String,
        description: Option<String>,
        describable: bool,
        focused: bool,
    },
    Table {
        id: u32,
        headers: Vec<TableCell>,
        rows: Vec<TableRow>,
        truncation: Option<TruncationInfo>,
        focused: bool,
    },
    List {
        id: u32,
        list_type: ListType,
        items: Vec<ListItem>,
        truncation: Option<TruncationInfo>,
        focused: bool,
    },
    Nav {
        id: u32,
        children: Vec<Element>,
        truncation: Option<TruncationInfo>,
        focused: bool,
    },
    Form {
        id: u32,
        action: Option<String>,
        children: Vec<Element>,
        truncation: Option<TruncationInfo>,
        focused: bool,
    },
    Section {
        id: u32,
        tag: String,
        children: Vec<Element>,
        truncation: Option<TruncationInfo>,
        focused: bool,
    },
    Iframe {
        id: u32,
        src: String,
        children: Vec<Element>,
        truncation: Option<TruncationInfo>,
        focused: bool,
    },
}

pub enum InputType {
    Text,
    Password,
    Email,
    Number,
    Tel,
    Url,
    Search,
    Textarea,
    Select,
    Checkbox,
    Radio,
    Date,
    File,
    Hidden,
    Other(String),
}

pub enum ListType {
    Ordered,
    Unordered,
    Description,
}

pub struct ListItem {
    pub children: Vec<Element>,
}

pub struct TableRow {
    pub cells: Vec<TableCell>,
}

pub struct TableCell {
    pub children: Vec<Element>,
}

pub struct TruncationInfo {
    pub shown: usize,
    pub total: usize,
}

pub struct ElementVisibility {
    pub present: bool,
    pub in_viewport: bool,
    pub obscured: bool,
    pub enabled: bool,
    pub visible: bool,
    pub clickable: bool,
}

pub enum MetadataRecord {
    Link {
        element_id: u32,
        text: String,
        href: String,
        focused: bool,
        visibility: Option<ElementVisibility>,
    },
    Image {
        element_id: u32,
        alt: String,
        src: String,
        description: Option<String>,
        describable: bool,
        focused: bool,
        visibility: Option<ElementVisibility>,
    },
    Button {
        element_id: u32,
        text: String,
        focused: bool,
        visibility: Option<ElementVisibility>,
    },
    Input {
        element_id: u32,
        name: Option<String>,
        input_type: InputType,
        placeholder: Option<String>,
        value: String,
        label: Option<String>,
        options: Vec<String>,
        focused: bool,
        visibility: Option<ElementVisibility>,
    },
    Table {
        element_id: u32,
        headers: usize,
        rows: usize,
        focused: bool,
        visibility: Option<ElementVisibility>,
    },
    List {
        element_id: u32,
        list_type: ListType,
        items: usize,
        focused: bool,
        visibility: Option<ElementVisibility>,
    },
    Nav {
        element_id: u32,
        children: usize,
        focused: bool,
        visibility: Option<ElementVisibility>,
    },
    Form {
        element_id: u32,
        action: Option<String>,
        children: usize,
        focused: bool,
        visibility: Option<ElementVisibility>,
    },
    Section {
        element_id: u32,
        tag: String,
        children: usize,
        focused: bool,
        visibility: Option<ElementVisibility>,
    },
    Iframe {
        element_id: u32,
        src: String,
        children: usize,
        focused: bool,
        visibility: Option<ElementVisibility>,
    },
}

pub struct ImageDescription {
    pub element_id: u32,
    pub alt: String,
    pub src: String,
    pub filename: Option<String>,
    pub description: String,
}
```

For a captured password input, `value` is empty when the field is empty and the fixed `[redacted]` marker when it is non-empty. Raw password values must not enter the page model, deferred metadata, YAML, or JSON output.

`Browser::launch(config)` creates a new detached session when `config.session.id` is `None`, and resumes an existing detached session when `config.session.id` is set, using Bowser's default file-backed `SessionStore`. Fresh Chrome processes choose their own ephemeral debugging port and publish it through a profile-local handshake. Resume validates the stored PID, exact profile, dynamic-port launch mode, and handshake port against the persisted endpoint before connecting; legacy fixed-port sessions retain exact-port validation.

`Browser::launch_with_store(config, store)` performs the same launch/resume flow with a caller-provided `Arc<dyn SessionStore>`. This allows detached-session metadata to live in a database or other persistence layer instead of the filesystem. Even when a custom store is used, `config.session.dir` still determines the default Chrome profile-root path when `user_data_dir` is not set.
