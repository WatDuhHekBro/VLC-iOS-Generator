use md5::{Digest, Md5};
use std::{
    fs,
    path::{Path, PathBuf},
};

// Step #1: Copy the file and rename it ("file:///.../song.mp3" --> "~/local/tmp/12.mp3")
// Step #2: Replace the entry with the index ("../Genre/song.mp3")

// Actually, forget using indexes, especially with multiple playlists. Use file hashes instead, no need to keep a database in memory.
// Step #1: Copy all target genres into "~/local/tmp/vlc-ios/<genre>/<hash>.mp3"
// Step #2: For all playlists in "<music-root>/[Playlists]/<playlist>.m3u8", replace file entries with "../<genre>/<hash>.mp3".

// For the program to work, you MUST add trailing slashes for MUSIC_ROOT!
const MUSIC_ROOT: &'static str = "/home/watduhhekbro/local/music/";
const TMP_ROOT: &'static str = "/home/watduhhekbro/local/tmp/vlc-ios";
const PLAYLISTS_FOLDER: &'static str = "[Playlists]";
// I just realized that you don't need to do this unless necessary.
const GENRES: [&str; 0] = [];
//const GENRES: [&str; 1] = ["unsorted-part2/slow"];

fn main() {
    // Loop through all the playlist files, generating transformed versions in the temporary root.
    let playlists_path = Path::new(MUSIC_ROOT).join(PLAYLISTS_FOLDER);
    let playlists_path_output = Path::new(TMP_ROOT).join(PLAYLISTS_FOLDER);
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

            let new_playlist_file = generate_transformed_playlist(&path.display().to_string());
            let new_playlist_path = playlists_path_output.join(filename);
            fs::write(new_playlist_path, new_playlist_file).ok();
        }
    }

    // Loop through all genre folders, copying and renaming into MD5 hashes
    for genre in GENRES {
        let src_path = Path::new(MUSIC_ROOT).join(genre);
        let out_path = Path::new(TMP_ROOT).join(genre);
        fs::create_dir_all(out_path.clone()).ok();

        // Copy each file in the src path to the out path, also renaming
        for entry in fs::read_dir(&src_path)
            .expect(&format!("Unknown genre path: \"{}\"!", src_path.display()))
        {
            let entry = entry.unwrap();
            let path = entry.path();
            let path = path.as_path();
            let hash = get_md5_of_file(&path.display().to_string());

            if path.is_dir() {
                continue;
            }

            fs::copy(path, out_path.join(format!("{hash}.mp3"))).ok();
        }
    }
}

fn generate_transformed_playlist(path: &String) -> String {
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
            let relative_filename = &filename.replace(MUSIC_ROOT, ""); // genre/C AllStar - lau haa.mp3
            let relative_parent = {
                let relative_parent = Path::new(relative_filename).parent();

                if let Some(relative_parent) = relative_parent {
                    relative_parent.display().to_string()
                } else {
                    String::new()
                }
            }; // genre

            let mut new_path = PathBuf::from("..");
            new_path.push(relative_parent);
            new_path.push(format!("{}.mp3", get_md5_of_file(filename)));

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