use crate::common::document::{DataSourceReference, Document};
use crate::common::search::{QueryResponse, QuerySource, SearchQuery};
use crate::common::traits::{SearchError, SearchSource};
use crate::local::LOCAL_QUERY_SOURCE_TYPE;
use async_trait::async_trait;
use base64::encode;
use dirs::data_dir;
use fuzzy_prefix_search::Trie;
use hostname;
use plist::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

pub struct ApplicationSearchSource {
    base_score: f64,
    app_dirs: Vec<PathBuf>,
    icons: HashMap<String, PathBuf>, // Map app names to their icon paths
    application_paths: fuzzy_prefix_search::Trie<String>, // Cached search locations
}

/// Extracts the app icon from the `.app` bundle or system icons and converts it to PNG format.
fn extract_icon_from_app_bundle(app_dir: &Path, app_data_folder: &Path) -> Option<PathBuf> {
    // First, check if the icon is specified in the info.plist (e.g., CFBundleIconFile)
    if let Some(icon_names) = get_icon_names_from_info_plist(app_dir) {
        for icon_name in icon_names {
            // Attempt to find the icon in the Resources folder
            let icns_path = app_dir.join(format!("Contents/Resources/{}", icon_name));

            if icns_path.exists() {
                // If the icon exists, convert it to PNG
                if let Some(output_path) =
                    convert_icns_to_png(&app_dir, &icns_path, app_data_folder)
                {
                    return Some(output_path);
                }
            } else {
                // If the icon name doesn't end with .icns, try appending it
                if !icon_name.ends_with(".icns") {
                    let icns_path_with_extension =
                        app_dir.join(format!("Contents/Resources/{}.icns", icon_name));
                    if icns_path_with_extension.exists() {
                        if let Some(output_path) = convert_icns_to_png(
                            &app_dir,
                            &icns_path_with_extension,
                            app_data_folder,
                        ) {
                            return Some(output_path);
                        }
                    }
                }
            }
        }
    }

    // Attempt to get the ICNS file from the app bundle (Contents/Resources/AppIcon.icns)
    if let Some(icon_path) = get_icns_from_app_bundle(app_dir) {
        if let Some(output_path) = convert_icns_to_png(&app_dir, &icon_path, app_data_folder) {
            return Some(output_path);
        }
    }

    // Fallback: Check for PNG icon in the Resources folder
    if let Some(png_icon_path) = get_png_from_resources(app_dir) {
        if let Some(output_path) = convert_png_to_png(&png_icon_path, app_data_folder) {
            return Some(output_path);
        }
    }

    // Fallback: If no icon found, return a default system icon
    if let Some(system_icon_path) = get_system_icon(app_dir) {
        if let Some(output_path) = convert_icns_to_png(&app_dir, &system_icon_path, app_data_folder)
        {
            return Some(output_path);
        }
    }

    None
}

fn get_icon_names_from_info_plist(app_dir: &Path) -> Option<Vec<String>> {
    let plist_path = app_dir.join("Contents/Info.plist");

    if plist_path.exists() {
        // Use `Value::from_file` directly, which parses the plist into a `Value` type
        if let Ok(plist_value) = Value::from_file(plist_path) {
            // Check if the plist value is a dictionary
            if let Some(icon_value) = plist_value.as_dictionary() {
                // Collect all icon-related keys
                let mut icons = Vec::new();

                // Check CFBundleIconFile
                if let Some(icon_file) = icon_value.get("CFBundleIconFile") {
                    if let Some(icon_name) = icon_file.as_string() {
                        icons.push(icon_name.to_string());
                    }
                }

                // Check CFBundleIconName (for default icon)
                if let Some(icon_name) = icon_value.get("CFBundleIconName") {
                    if let Some(name) = icon_name.as_string() {
                        icons.push(name.to_string());
                    }
                }

                // Check CFBundleTypeIconFile
                if let Some(type_icon_file) = icon_value.get("CFBundleTypeIconFile") {
                    if let Some(icon_name) = type_icon_file.as_string() {
                        icons.push(icon_name.to_string());
                    }
                }

                // If there are any icons found, return them
                if !icons.is_empty() {
                    return Some(icons);
                }
            }
        }
    }
    None
}

/// Tries to get the ICNS icon from the `.app` bundle.
fn get_icns_from_app_bundle(app_dir: &Path) -> Option<PathBuf> {
    let icns_path = app_dir.join("Contents/Resources/AppIcon.icns");
    if icns_path.exists() {
        Some(icns_path)
    } else {
        None
    }
}

/// Tries to get a PNG icon from the `.app` bundle's Resources folder.
fn get_png_from_resources(app_dir: &Path) -> Option<PathBuf> {
    let png_path = app_dir.join("Contents/Resources/Icon.png");
    if png_path.exists() {
        Some(png_path)
    } else {
        None
    }
}

/// Converts an ICNS file to PNG using macOS's `sips` command.
fn convert_icns_to_png(
    app_dir: &Path,
    icns_path: &Path,
    app_data_folder: &Path,
) -> Option<PathBuf> {
    if let Some(app_name) = app_dir.file_name().and_then(|name| name.to_str()) {
        let icon_storage_dir = app_data_folder.join("coco-appIcons");
        fs::create_dir_all(&icon_storage_dir).ok();

        let output_png_path = icon_storage_dir.join(format!("{}.png", app_name));

        if output_png_path.exists() {
            return Some(output_png_path);
        }

        // dbg!("Converting ICNS to PNG:", &output_png_path);

        // Run the `sips` command to convert the ICNS to PNG
        let status = Command::new("sips")
            .arg("-s")
            .arg("format")
            .arg("png")
            .arg(icns_path)
            .arg("--out")
            .arg(&output_png_path)
            .stdout(Stdio::null()) // Redirect stdout to null
            .stderr(Stdio::null()) // Redirect stderr to null
            .status();

        if let Ok(status) = status {
            if status.success() {
                return Some(output_png_path);
            }
        } else {
            dbg!("Failed to convert ICNS to PNG:", &output_png_path);
        }
    }
    None
}

/// Converts a PNG file to PNG (essentially just copying it to a new location).
fn convert_png_to_png(png_path: &Path, app_data_folder: &Path) -> Option<PathBuf> {
    if let Some(app_name) = png_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|name| name.to_str())
    {
        let icon_storage_dir = app_data_folder.join("coco-appIcons");
        fs::create_dir_all(&icon_storage_dir).ok();

        let output_png_path = icon_storage_dir.join(format!("{}.png", app_name));

        // Copy the PNG file to the output directory
        if let Err(_e) = fs::copy(png_path, &output_png_path) {
            return None;
        }

        return Some(output_png_path);
    }
    None
}

/// Fallback function to fetch a system icon if the app doesn't have its own.
fn get_system_icon(_app_dir: &Path) -> Option<PathBuf> {
    // Just a placeholder for getting a default icon if no app-specific icon is found
    let default_icon_path = Path::new("/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/GenericApplicationIcon.icns");

    if default_icon_path.exists() {
        Some(default_icon_path.to_path_buf())
    } else {
        None
    }
}

impl ApplicationSearchSource {
    pub fn new(base_score: f64, app_dirs: Vec<PathBuf>) -> Self {
        let mut icons = HashMap::new();

        // Collect search locations as strings
        let applications = Trie::new();

        // Iterate over the directories to find .app files and extract icons
        for app_dir in &app_dirs {
            // Use WalkDir to recursively get all files in app_dir
            for entry in WalkDir::new(app_dir).into_iter().filter_map(Result::ok) {
                let file_path = entry.path();
                if file_path.is_dir() && file_path.extension() == Some("app".as_ref()) {
                    if let Some(app_data_folder) = data_dir() {
                        let file_path_str = file_path.to_string_lossy().to_string(); // Convert to owned String if needed
                        if file_path
                            .parent()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .contains(".app/Contents/")
                        {
                            continue;
                        }
                        let search_word = file_path
                            .file_name()
                            .unwrap() // unwrap() might panic if there's no file name
                            .to_str()
                            .unwrap() // unwrap() might panic if it's not valid UTF-8
                            .trim_end_matches(".app")
                            .to_lowercase(); // to_lowercase returns a String, which is owned

                        //TODO, replace this hard-coded name to actual local app name in case it may change
                        if search_word.is_empty() || search_word.eq("coco-ai") {
                            continue;
                        }

                        let search_word_ref = search_word.as_str(); // Get a reference to the string slice
                        applications.insert(search_word_ref, file_path_str.clone());
                        if let Some(icon_path) =
                            extract_icon_from_app_bundle(&file_path, &app_data_folder)
                        {
                            icons.insert(file_path_str, icon_path);
                        } else {
                            dbg!("No icon found for:", &file_path);
                        }
                    }
                }
            }
        }

        ApplicationSearchSource {
            base_score,
            app_dirs,
            icons,
            application_paths: applications,
        }
    }
}

/// Extracts the clean app name by removing `.app`
fn clean_app_name(path: &Path) -> Option<String> {
    path.file_name()?
        .to_str()
        .map(|name| name.trim_end_matches(".app").to_string())
}

#[async_trait]
impl SearchSource for ApplicationSearchSource {
    fn get_type(&self) -> QuerySource {
        QuerySource {
            r#type: LOCAL_QUERY_SOURCE_TYPE.into(),
            name: hostname::get()
                .unwrap_or("My Computer".into())
                .to_string_lossy()
                .into(),
            id: "local_applications".into(),
        }
    }

    async fn search(&self, query: SearchQuery) -> Result<QueryResponse, SearchError> {
        let query_string = query
            .query_strings
            .get("query")
            .unwrap_or(&"".to_string())
            .to_lowercase();

        if query_string.is_empty() {
            return Ok(QueryResponse {
                source: self.get_type(),
                hits: Vec::new(),
                total_hits: 0,
            });
        }

        let mut total_hits = 0;
        let mut hits = Vec::new();

        let mut results = self
            .application_paths
            .search_within_distance_scored(&query_string, 3);

        // Check for NaN or extreme score values and handle them properly
        results.sort_by(|a, b| {
            // If either score is NaN, consider them equal (you can customize this logic as needed)
            if a.score.is_nan() || b.score.is_nan() {
                std::cmp::Ordering::Equal
            } else {
                // Otherwise, compare the scores as usual
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        if !results.is_empty() {
            for result in results {
                let file_name_str = result.word;
                let file_path_str = result.data.get(0).unwrap().to_string();
                let file_path = PathBuf::from(file_path_str.clone());
                let cleaned_file_name = clean_app_name(&file_path).unwrap();
                total_hits += 1;
                let mut doc = Document::new(
                    Some(DataSourceReference {
                        r#type: Some(LOCAL_QUERY_SOURCE_TYPE.into()),
                        name: Some("Applications".into()),
                        id: Some(file_name_str.clone()),
                    }),
                    file_path_str.clone(),
                    "Application".to_string(),
                    cleaned_file_name,
                    file_path_str.clone(),
                );

                // Attach icon if available
                if let Some(icon_path) = self.icons.get(file_path_str.as_str()) {
                    // doc.icon = Some(format!("file://{}", icon_path.to_string_lossy()));
                    // dbg!(&doc.icon);
                    if let Ok(icon_data) = read_icon_and_encode(icon_path) {
                        doc.icon = Some(format!("data:image/png;base64,{}", icon_data));
                    }
                }

                hits.push((doc, self.base_score + result.score as f64));
            }
        }

        Ok(QueryResponse {
            source: self.get_type(),
            hits,
            total_hits,
        })
    }
}

// Function to read the icon file and convert it to base64
fn read_icon_and_encode(icon_path: &Path) -> Result<String, std::io::Error> {
    // Read the icon file as binary data
    let icon_data = fs::read(icon_path)?;

    // Encode the data to base64
    Ok(encode(&icon_data))
}
