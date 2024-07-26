use dotenvy::dotenv;
use md5::{Digest, Md5};
use std::env;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

// Actually, forget using indexes, especially with multiple playlists. Use file hashes instead, no need to keep a database in memory.
// Step #1: Copy all target genres into "~/local/tmp/vlc-ios/<genre>/<hash>.mp3"
// Step #2: For all playlists in "<music-root>/[Playlists]/<playlist>.m3u8", replace file entries with "../<genre>/<hash>.mp3".

fn main() {
    // Command line arguments
    let args: Vec<String> = env::args().collect();
    let genres = &args[1..];

    // Load .env
    dotenv().ok();

    // Make sure all variables exist
    let env_music_root = env::var("MUSIC_ROOT").expect("No MUSIC_ROOT defined!");
    let env_tmp_root = env::var("TMP_ROOT").expect("No TMP_ROOT defined!");
    let env_playlists_folder = env::var("PLAYLISTS_FOLDER").unwrap_or(String::from("[Playlists]"));

    // Setup MD5 hash cache <"/home/...", "1234567890abcdef">
    let mut cache: HashMap<String, String> = HashMap::new();
    /*cache.insert(
        "/home/watduhhekbro/local/music/Slow/엠씨더맥스 (M.C the MAX) - 넘쳐흘러.mp3".into(),
        "test-hash".into(),
    );*/

    // Loop through all genre folders, copying and renaming into MD5 hashes
    for genre in genres {
        let src_path = Path::new(&env_music_root).join(genre);
        let out_path = Path::new(&env_tmp_root).join(genre);
        fs::create_dir_all(out_path.clone()).ok();

        // Copy each file in the src path to the out path, also renaming
        for entry in fs::read_dir(&src_path)
            .expect(&format!("Unknown genre path: \"{}\"!", src_path.display()))
        {
            let entry = entry.unwrap();
            let path = entry.path();
            let path = path.as_path();
            let path_as_str = path.display().to_string();

            if path.is_dir() {
                continue;
            }

            let stored_hash = cache.get(&path_as_str);

            if let Some(stored_hash) = stored_hash {
                fs::copy(path, out_path.join(format!("{stored_hash}.mp3"))).ok();
            } else {
                let hash = get_md5_of_file(&path_as_str);
                let filename = format!("{hash}.mp3");

                // Update the cache for future iterations
                cache.insert(path_as_str, hash);

                fs::copy(path, out_path.join(filename)).ok();
            }
        }
    }

    // Loop through all the playlist files, generating transformed versions in the temporary root.
    let playlists_path = Path::new(&env_music_root).join(&env_playlists_folder);
    let playlists_path_output = Path::new(&env_tmp_root).join(&env_playlists_folder);
    fs::create_dir_all(&playlists_path_output).ok();

    for entry in fs::read_dir(playlists_path).expect("Playlists folder doesn't exist!") {
        let entry = entry.unwrap();
        let path = entry.path();
        let path = path.as_path();
        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        let extension = path.extension();

        if let Some(extension) = extension {
            if extension != "m3u8" {
                continue;
            }

            let new_playlist_file = generate_transformed_playlist(
                &path.display().to_string(),
                &env_music_root,
                &mut cache,
            );
            let new_playlist_path = playlists_path_output.join(filename);
            fs::write(new_playlist_path, new_playlist_file).ok();
        }
    }
}

fn generate_transformed_playlist(
    path: &String,
    env_music_root: &String,
    cache: &mut HashMap<String, String>,
) -> String {
    let file = fs::read_to_string(path).expect(&format!("Invalid path: \"{path}\"!"));
    let mut output = String::new();

    for line in file.lines() {
        // Keep the line
        if line.starts_with("#") {
            output.push_str(line);
            output.push('\n');
        }
        // If the line is a path (assumed), then change the file reference to an MD5 hash
        else {
            let filename = &decode_uri_component(&line.to_string()).replace("file://", ""); // /home/watduhhekbro/local/music/genre/C AllStar - lau haa.mp3
            let relative_filename = &filename.replace(env_music_root, ""); // genre/C AllStar - lau haa.mp3
            let relative_parent = {
                let relative_parent = Path::new(relative_filename).parent();

                if let Some(relative_parent) = relative_parent {
                    relative_parent.display().to_string()
                } else {
                    String::new()
                }
            }; // genre

            // First check if the filename associated already has its hash calculated
            let stored_hash = cache.get(filename);

            let new_filename = if let Some(stored_hash) = stored_hash {
                format!("{stored_hash}.mp3")
            } else {
                let hash = get_md5_of_file(filename);
                let output = format!("{hash}.mp3");

                // Make sure to add the result to the cache
                cache.insert(filename.clone(), hash);

                output
            };

            let mut new_path = PathBuf::from("..");
            new_path.push(relative_parent);
            new_path.push(new_filename);

            output.push_str(&new_path.display().to_string());
            output.push('\n');
        }
    }

    output
}

fn get_md5_of_file(path: &String) -> String {
    let mut hasher = Md5::new();
    let bytes = fs::read(path).unwrap();
    hasher.update(bytes);
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn decode_uri_component(text: &String) -> String {
    urlencoding::decode(text)
        .expect(&format!(
            "The string \"{text}\" was not a valid URI component!"
        ))
        .to_string()
}
