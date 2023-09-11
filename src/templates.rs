pub const APP_BUILD_TEMPLATE: &str = r#"
"AppBuild"
\{
	"AppID" "{steam_app_id}" // Your AppID
	"Desc" "Lowly build" // internal description for this build
	"SetLive" "{steam_branch}" // set this build live on beta branch AlphaTest
	"ContentRoot" "{game_output_path}" // content root folder relative to this script file
	"Depots"
	\{
		// file mapping instructions for each depot are in separate script files
		"{steam_depot_id}" "lowly_depot_build.vdf"
	}
}
"#;

pub const APP_DEPOT_BUILD_TEMPLATE: &str = r#"
"DepotBuild"
\{
	// Set your assigned depot ID here
	"DepotID" "{steam_depot_id}"

	// include all files recursivley
	"FileMapping"
	\{
		// This can be a full path, or a path relative to ContentRoot
		"LocalPath" "*"

		// This is a path relative to the install folder of your game
		"DepotPath" "."
		
		// If LocalPath contains wildcards, setting this means that all
		// matching files within subdirectories of LocalPath will also
		// be included.
		"Recursive" "1"
  }
}
"#;