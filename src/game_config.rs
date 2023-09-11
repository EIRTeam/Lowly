use serde_derive::{Deserialize, Serialize};
use tinytemplate::TinyTemplate;
use crate::templates;
#[derive(Debug, Deserialize, Serialize)]
pub struct GodotPCKInfo {
    pub name: String,
    pub export_preset_name: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GodotGameInfoExtraFile {
    pub from: String,
    pub to: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GodotGameInfo {
    pub steam_app_id: i32,
    pub steam_depot_id: i32,
    pub steam_branch: String,
    pub steam_account_name: String,

    pub engine_binaries_path: String,

    #[serde(default)]
    pub extra_files: Vec<GodotGameInfoExtraFile>,

    pub editor_path: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub pcks: Vec<GodotPCKInfo>,
}

#[derive(Serialize)]
pub struct GodotBuildContext<'a> {
    #[serde(skip_serializing)]
    tiny_template: TinyTemplate<'a>,
    #[serde(flatten)]
    pub game_info: GodotGameInfo,
    pub game_path: String,
    pub game_output_path: String
}

impl GodotBuildContext<'_> {
    pub fn new(game_info: GodotGameInfo, game_path: String, game_output_path: String) -> Result<Self, tinytemplate::error::Error> {
        let mut tiny_template = TinyTemplate::new();
        tiny_template.add_template("app_build", templates::APP_BUILD_TEMPLATE)?;
        tiny_template.add_template("depot_build", templates::APP_DEPOT_BUILD_TEMPLATE)?;    
        Ok(GodotBuildContext {
            tiny_template,
            game_info,
            game_path,
            game_output_path
        })
    }
    pub fn compile_app_build(&self) -> Result<String, tinytemplate::error::Error> {
        self.tiny_template.render("app_build", self)
    }
    pub fn compile_depot_build(&self) -> Result<String, tinytemplate::error::Error> {
        self.tiny_template.render("depot_build", self)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "game_type")]
pub enum GameInfoType {
    Godot(GodotGameInfo)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GameConfig {
    #[serde(flatten)]
    pub game_info: GameInfoType
}
