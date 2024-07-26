# Overview

**Problem:** Importing `m3u8` playlists VLC on iOS has an issue where certain Unicode characters (like Korean/Vietnamese characters) fail to link correctly to imported playlists, causing all sorts of issues. This program automates the workaround process of generating new files with safe filenames via MD5 hashes and making sure playlists have relative paths to the music.

The program always scans your playlists folder and generates new suitable playlist files. Command line arguments are read as a list of genre folders to copy, e.g. `program "Genre1" "Path/To/Genre2"` generates those genres first before generating playlist files.

# Usage

`.env` file required!

- `MUSIC_ROOT`: `/home/watduhhekbro/local/music/` (**NOTE:** The leading slash required or the program will break!)
- `TMP_ROOT`: `/home/watduhhekbro/local/tmp/vlc-ios`
- `PLAYLISTS_FOLDER` (optional): Defaults to `[Playlists]`
