use clap::Parser;
use config::{ Config, File };
mod templates;
mod game_config;
use game_config::{ GameConfig, GameInfoType, GodotGameInfo, GodotBuildContext };
use std::{io::{BufWriter, Write}, process::Command, time::{SystemTime, UNIX_EPOCH}};
use serde_derive::Serialize;

use std::path::Path;
use tinytemplate::TinyTemplate;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the game's project
    #[arg(short, long)]
    path: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long, default_value = "staging")]
    steam_branch: String,

    #[arg(short, long, default_value_t = false)]
    dry_run: bool,

    #[arg(long, default_value_t = false)]
    publish_only: bool,

    #[arg(long, default_value = ".lowly.toml")]
    config_path: String
}

#[derive(Serialize)]
struct GameVersionInfo {
    build_time: u64,
    commit_hash: String,
    is_dirty: bool
}

fn main() -> Result<(), Box<dyn std::error::Error>> { 
    let args = Args::parse();

    let game_path = Path::new(&args.path);
    let game_config_path = game_path.join(args.config_path);
    
    let cwd = std::env::current_dir()?;
    let game_config_path_local = cwd.join(".lowly_local.toml");

    let game_config: GameConfig = Config::builder()
    .add_source(File::with_name(&game_config_path.to_string_lossy()))
    .add_source(File::with_name(&game_config_path_local.to_string_lossy()).required(false))
    .build()?.try_deserialize()?;

    let output_path_abs = std::fs::canonicalize(Path::new(&args.output))?;
    let repo = gix::discover(game_path);
    if let Ok(repo) = repo {
        let head = repo.head()?;
        let version_info = GameVersionInfo {
            commit_hash: head.id().unwrap().shorten_or_id().to_string(),
            build_time: SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs(),
            is_dirty: repo.is_dirty().expect("Something went wrong")
        };

        let version_data_path = game_path.join("version.json");
        let file = std::fs::File::create(version_data_path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &version_info)?;
        writer.flush()?;
    }

    match game_config.game_info {
        GameInfoType::Godot(godot_game_info) => {
            godot(godot_game_info, game_path.to_string_lossy().to_string(), output_path_abs.to_string_lossy().to_string())?;
        }
    }

    Ok(())
}

#[derive(Serialize)]
struct GodotTemplateInfo {
    #[serde(flatten)]
    pub game_info: GodotGameInfo,
    pub game_path: String,
    pub game_output_path: String
}

fn godot(game_info: GodotGameInfo, game_path: String, game_output_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let build_context = GodotBuildContext::new(game_info, game_path, game_output_path)?;

    for pck in build_context.game_info.pcks.iter() {
        let path = Path::new(&build_context.game_output_path).join(&pck.name);
        let status = Command::new(&build_context.game_info.editor_path)
            .arg("--headless")
            .arg("--path")
            .arg(&build_context.game_path)
            .arg("--export-pack")
            .arg(&pck.export_preset_name)
            .arg(path)
            .status()?;
        if !status.success() {
            println!("Godot process failed with error code {}", status);
        }
    }

    for file in build_context.game_info.extra_files.iter() {
        let mut tt_file = TinyTemplate::new();
        tt_file.add_template("from", &file.from)?;
        tt_file.add_template("to", &file.to)?;

        let from_path = tt_file.render("from", &build_context)?;
        let to_path = tt_file.render("to", &build_context)?;
        
        let from = Path::new(&from_path);
        let to = Path::new(&to_path);

        std::fs::copy(from, to).expect("error copying");
    }

    let temp_dir = std::env::temp_dir();
    let temp_dir_path = Path::new(&temp_dir);

    let build_path = temp_dir_path.join("lowly_build.vdf");
    std::fs::write(&build_path, build_context.compile_app_build()?).expect("Error writing app build script");
    std::fs::write(temp_dir_path.join("lowly_depot_build.vdf"), build_context.compile_depot_build()?).expect("Error writing app depot script");

    let status = Command::new("steamcmd")
        .arg("+login")
        .arg(build_context.game_info.steam_account_name)
        .arg("+run_app_build")
        .arg(build_path)
        .arg("+exit")
        .status()?;
    if !status.success() {
        println!("SteamCMD process failed with error code {}", status);
    }
    println!("{}", build_context.game_info.steam_branch);

    println!("Lowly: All OK!");

    Ok(())
}
