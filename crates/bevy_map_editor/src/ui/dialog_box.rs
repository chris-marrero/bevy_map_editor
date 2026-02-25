use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_egui::{EguiContext, PrimaryEguiContext};
use egui_async::{Bind, EguiAsyncPlugin};
use rfd::{AsyncFileDialog, FileHandle};
use std::path::PathBuf;

pub struct DialogBoxPlugin;

impl Plugin for DialogBoxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DialogBinds>();
        app.world_mut()
            .register_component_hooks::<PrimaryEguiContext>()
            .on_add(|mut world: DeferredWorld, hook_context: HookContext| {
                let mut egui_context = world.get_mut::<EguiContext>(hook_context.entity).unwrap();
                egui_context.get_mut().add_plugin(EguiAsyncPlugin);
            });
    }
}

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
                DialogType::Open.into(),
                DialogType::OpenSpritesheet.into(),
                DialogType::SaveAs.into(),
                DialogType::NewTilesetImage.into(),
                DialogType::AddImageToTileset.into(),
                DialogType::ParentDirectory.into(),
                DialogType::NewProject.into(),
                DialogType::VsCode.into(),
                DialogType::Icon.into(),
            ],
            set_directory: None,
            set_file_name: None,
            set_title: None,
        }
    }
}

impl DialogBinds {
    pub fn spawn_and_poll(&mut self, dialog: DialogType) -> DialogStatus {
        let mut file_dialog = AsyncFileDialog::new();

        for &(file_name, extensions) in dialog.filter() {
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

        let bind = self.get_bind(dialog);
        if let Some(file) = bind.read_or_request(|| async move { dialog.open(file_dialog).await }) {
            let status = if let Ok(path) = file.clone().map(|f| f.path().to_path_buf()) {
                DialogStatus::Success(path)
            } else {
                DialogStatus::Cancel
            };

            bind.clear();

            status
        } else {
            DialogStatus::Pending
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

    pub fn in_progress(&mut self, dialog: DialogType) -> bool {
        let state = self.get_bind(dialog).get_state();
        state == egui_async::State::Pending || state == egui_async::State::Finished
    }

    fn get_bind(&mut self, dialog: DialogType) -> &mut Bind<FileHandle, ()> {
        self.binds
            .iter_mut()
            .find_map(|bind| {
                if dialog == bind.dialog_type {
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
    dialog_type: DialogType,
    bind: Bind<FileHandle, ()>,
}

pub enum DialogStatus {
    Success(PathBuf),
    Pending,
    Cancel,
}

impl DialogBind {
    fn new(dialog_type: DialogType) -> Self {
        Self {
            bind: default(),
            dialog_type,
        }
    }
}

impl From<DialogType> for DialogBind {
    fn from(dialog_type: DialogType) -> Self {
        Self::new(dialog_type)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DialogType {
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

impl DialogType {
    fn filter(self) -> &'static [FilterPair] {
        match self {
            DialogType::Open => MAP_PROJECT,
            DialogType::SaveAs => MAP_PROJECT,
            DialogType::NewTilesetImage => IMAGE,
            DialogType::AddImageToTileset => IMAGE,
            DialogType::OpenSpritesheet => SPRITESHEET,
            DialogType::ParentDirectory => &[],
            DialogType::VsCode => EXECUTABLE,
            DialogType::NewProject => MAP_PROJECT,
            DialogType::Icon => ICON,
        }
    }

    async fn open(self, file_dialog: AsyncFileDialog) -> Result<FileHandle, ()> {
        match self {
            DialogType::Open => file_dialog.pick_file().await,
            DialogType::SaveAs => file_dialog.save_file().await,
            DialogType::NewTilesetImage => file_dialog.pick_file().await,
            DialogType::AddImageToTileset => file_dialog.pick_file().await,
            DialogType::OpenSpritesheet => file_dialog.pick_file().await,
            DialogType::ParentDirectory => file_dialog.pick_folder().await,
            DialogType::VsCode => file_dialog.pick_file().await,
            DialogType::NewProject => file_dialog.save_file().await,
            DialogType::Icon => file_dialog.pick_file().await,
        }
        .ok_or(())
    }
}
