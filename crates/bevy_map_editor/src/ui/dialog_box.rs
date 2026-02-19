use bevy::prelude::*;
use egui_async::Bind;
use rfd::{AsyncFileDialog, FileHandle};
use std::path::PathBuf;

#[derive(Resource)]
pub struct DialogBinds {
    binds: Vec<DialogBind>,
    set_directory: Option<PathBuf>,
    set_file_name: Option<String>,
    set_title: Option<String>,
}

impl Default for DialogBinds {
    fn default() -> Self {
        Self {
            binds: vec![
                DialogKind::Open.into(),
                DialogKind::OpenSpritesheet.into(),
                DialogKind::SaveAs.into(),
                DialogKind::NewTilesetImage.into(),
                DialogKind::AddImageToTileset.into(),
                DialogKind::ParentDirectory.into(),
                DialogKind::NewProject.into(),
                DialogKind::VsCode.into(),
                DialogKind::Icon.into(),
            ],
            set_directory: None,
            set_file_name: None,
            set_title: None,
        }
    }
}

impl DialogBinds {
    pub fn spawn_and_poll(&mut self, kind: DialogKind) -> Option<PathBuf> {
        let mut file_dialog = AsyncFileDialog::new();

        for &(file_name, extensions) in kind.filter() {
            file_dialog = file_dialog.add_filter(file_name, extensions)
        }

        if let Some(start_dir) = self.set_directory.take() {
            file_dialog = file_dialog.set_directory(start_dir);
        }

        if let Some(file_name) = self.set_file_name.take() {
            file_dialog = file_dialog.set_file_name(file_name);
        }

        if let Some(title) = self.set_title.take() {
            file_dialog = file_dialog.set_title(title);
        }

        let bind = self.get_bind(kind);
        if let Some(file) =
            bind.read_or_request(|| async move { kind.open(file_dialog).await.ok_or(()) })
        {
            let r = file.clone().map(|file| file.path().to_path_buf()).ok().clone();
            bind.clear();
            r
        } else {
            None
        }
    }

    pub fn set_directory(&mut self, path: PathBuf) -> &mut Self {
        self.set_directory = Some(path);
        self
    }

    pub fn set_file_name(&mut self, file_name: String) -> &mut Self {
        self.set_file_name = Some(file_name);
        self
    }

    pub fn set_title(&mut self, title: &str) -> &mut Self {
        self.set_title = Some(title.to_string());
        self
    }

    pub fn in_progress(&mut self, kind: DialogKind) -> bool {
        let state = self.get_bind(kind).get_state();
        state == egui_async::State::Pending || state == egui_async::State::Finished
    }

    fn get_bind(&mut self, kind: DialogKind) -> &mut Bind<FileHandle, ()> {
        self.binds
            .iter_mut()
            .find_map(|bind| {
                if kind == bind.kind {
                    Some(&mut bind.bind)
                } else {
                    None
                }
            })
            .unwrap()
    }
}

#[derive(Debug)]
struct DialogBind {
    kind: DialogKind,
    bind: Bind<FileHandle, ()>,
}

impl DialogBind {
    fn new(kind: DialogKind) -> Self {
        Self {
            bind: default(),
            kind,
        }
    }
}

impl From<DialogKind> for DialogBind {
    fn from(kind: DialogKind) -> Self {
        Self::new(kind)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DialogKind {
    Open,
    OpenSpritesheet,
    SaveAs,
    NewTilesetImage,
    AddImageToTileset,
    ParentDirectory,
    NewProject,
    VsCode,
    Icon,
}

type FilterPair = (&'static str, &'static [&'static str]);

static MAP_PROJECT: &[FilterPair] = &[("Map Project", &["map.json", "json"])];
static IMAGE: &[FilterPair] = &[("Images", &["png", "jpg", "jpeg"])];
static SPRITESHEET: &[FilterPair] = &[("Images", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])];
static EXECUTABLE: &[FilterPair] = &[("Executable", &["exe"])];
static ICON: &[FilterPair] = &[
    ("Image Files", &["png", "jpg", "jpeg", "bmp", "gif", "svg"]),
    ("All Files", &["*"]),
];

impl DialogKind {
    fn filter(self) -> &'static [FilterPair] {
        match self {
            DialogKind::Open => MAP_PROJECT,
            DialogKind::SaveAs => MAP_PROJECT,
            DialogKind::NewTilesetImage => IMAGE,
            DialogKind::AddImageToTileset => IMAGE,
            DialogKind::OpenSpritesheet => SPRITESHEET,
            DialogKind::ParentDirectory => &[],
            DialogKind::VsCode => EXECUTABLE,
            DialogKind::NewProject => MAP_PROJECT,
            DialogKind::Icon => ICON,
        }
    }

    async fn open(self, file_dialog: AsyncFileDialog) -> Option<FileHandle> {
        match self {
            DialogKind::Open => file_dialog.pick_file().await,
            DialogKind::SaveAs => file_dialog.save_file().await,
            DialogKind::NewTilesetImage => file_dialog.pick_file().await,
            DialogKind::AddImageToTileset => file_dialog.pick_file().await,
            DialogKind::OpenSpritesheet => file_dialog.pick_file().await,
            DialogKind::ParentDirectory => file_dialog.pick_folder().await,
            DialogKind::VsCode => file_dialog.pick_file().await,
            DialogKind::NewProject => file_dialog.save_file().await,
            DialogKind::Icon => file_dialog.pick_file().await,
        }
    }
}
