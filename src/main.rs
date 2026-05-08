use clearscreen::clear;
use console::style;
use std::{
    fs,
    io::{self, Write, stdout},
    path::PathBuf,
    thread,
    time::{self, Duration},
};

use anyhow::{Context, Result, anyhow};

const GAME_ID: u32 = 4_000;
const ADDON_NAME: &str = "ChangerMusicLanRP";
const NEEDED_HZ: u32 = 44_100;
const CREDITS: &str = "written in Rust with ❤️  by sekta; Original by rty000pro";
const CATEGORIES: &[&str] = &["calm", "epic", "other", "tense"];

//
//
// Set Window Title
//
//

fn set_app_title(title: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = winconsole::console::set_title(title);
    }

    #[cfg(not(target_os = "windows"))]
    {
        print!("\x1B]0;{}\x07", title);
    }
}

//
//
// Steam&Game Path
//
//

/// Возвращает путь к корневой директории Gmod а так же к папке библиотеки стима в которой находится игра, при ошибке или если игры нет, возвращает None
fn get_game_path() -> Option<(PathBuf, PathBuf)> {
    let steam_dir = steamlocate::locate().ok()?; // .ok() при результате просто сохранит его в steam_dir, при ошибке вся функция вернёт None
    let (garrys_mod, lib) = steam_dir.find_app(GAME_ID).ok()??;

    let path = lib
        .path()
        .to_path_buf()
        .join("steamapps")
        .join("common")
        .join(garrys_mod.install_dir);

    Some((path, lib.path().to_path_buf()))
}

//
//
// Addon Folders Management
//
//

fn manage_addon_folders() {
    let Some((game_path, _lib)) = get_game_path() else {
        return;
    };

    let addon_path = game_path.join("garrysmod").join("addons").join(ADDON_NAME);
    let addon_folder_exists = match fs::exists(&addon_path) {
        Ok(bool) => bool,
        Err(_) => false,
    };

    if addon_folder_exists == true {
        delete_addon_folder(&addon_path);
        return;
    }

    let music_path = addon_path.join("sound").join("lanrp").join("music");

    for cat in CATEGORIES {
        let cat_path = music_path.join(cat);
        if let Err(err) = fs::create_dir_all(cat_path) {
            println!("Failed to create dir for category {cat}: {err}");
            continue;
        }
    }

    println!("Addon folder created! Returning to menu...");
    thread::sleep(time::Duration::from_secs(1));
}

fn delete_addon_folder(addon_path: &PathBuf) {
    print!("You sure to delete addon folder? (y/n): ");
    let _ = stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return;
    }

    match input.trim() {
        "y" => match fs::remove_dir_all(addon_path) {
            Ok(()) => {
                println!("Addon folder deleted, we hope you saved all tracks...");
                thread::sleep(time::Duration::from_secs(1));
            }
            Err(err) => {
                println!("Error occured when deleting addon folder: {err}");
                thread::sleep(time::Duration::from_secs(3));
            }
        },
        _ => return,
    }
}

/// Возвращает true если папка аддона существует, иначе false
fn is_addon_folder_exists() -> bool {
    let Some((game_path, _lib)) = get_game_path() else {
        return false;
    };

    let addon_path = game_path.join("garrysmod").join("addons").join(ADDON_NAME);
    return match fs::exists(&addon_path) {
        Ok(bool) => bool,
        Err(_) => false,
    };
}

//
//
// Adding music
//
//

fn add_music_api(path: PathBuf, category: &str) {
    let Some((game_path, _lib)) = get_game_path() else {
        return;
    };
    let mut full_path = game_path
        .join("garrysmod")
        .join("addons")
        .join(ADDON_NAME)
        .join("sound")
        .join("lanrp")
        .join("music")
        .join(category);

    let file_name = path.file_name().unwrap().display().to_string();
    full_path = full_path.join(file_name);

    match fs::copy(path, full_path) {
        Ok(_) => {
            println!("Song added to list...");
            thread::sleep(time::Duration::from_secs(1));
        }
        Err(err) => {
            println!("Error occured when adding copying song: {err}");
            thread::sleep(time::Duration::from_secs(1));
        }
    };
}

fn add_music() {
    // Check

    if !is_addon_folder_exists() {
        println!("Addon folder not found. Create it in the menu...");
        thread::sleep(Duration::from_secs(3));
        return;
    }

    let _ = set_app_title("lanrp-music-rs - add music");

    // Cats

    let mut cats = String::new();

    let mut i = 0;
    for s in CATEGORIES.iter() {
        cats = cats
            + s
            + (if i == (CATEGORIES.iter().count() - 1) {
                ""
            } else {
                "/"
            });
        i = i + 1;
    }

    println!("Choose mp3 file..."); // wav/ogg

    let retr = || {
        println!("File not picked, returning to menu...");
        thread::sleep(time::Duration::from_secs(1));
        return;
    };

    let fileopt = rfd::FileDialog::new()
        .add_filter("sound/music", &["mp3"]) // , "ogg", "wav"
        .pick_file();

    match fileopt {
        Some(pathbuf) => {
            print!("Enter category({cats}): ");
            stdout().flush().unwrap();

            let mut input_category = String::new();
            if io::stdin().read_line(&mut input_category).is_err() {
                return;
            }

            add_music_api(pathbuf, &input_category.trim())
        }
        None => retr(),
    }
}

//
//
// Deleting music
//
//

fn delete_music_api(name: &str, category: &str) -> Result<()> {
    let Some((game_path, _lib)) = get_game_path() else {
        // Возвращаем ошибку с текстом прямо здесь
        return Err(anyhow!("Garry's Mod not found! Cannot delete file."));
    };

    let full_path = game_path
        .join("garrysmod")
        .join("addons")
        .join(ADDON_NAME)
        .join("sound")
        .join("lanrp")
        .join("music")
        .join(category)
        .join(name);

    fs::remove_file(full_path).context("Failed to remove music file")?;

    Ok(())
}

fn delete_music() {
    // Check

    if !is_addon_folder_exists() {
        println!("Addon folder not found. Create it in the menu...");
        thread::sleep(Duration::from_secs(3));
        return;
    }

    let _ = set_app_title("lanrp-music-rs - delete music");

    // Category

    let mut cats = String::new();

    let mut i = 0;
    for s in CATEGORIES.iter() {
        cats = cats
            + s
            + (if i == (CATEGORIES.iter().count() - 1) {
                ""
            } else {
                "/"
            });
        i = i + 1;
    }

    print!("Enter category(ex. {cats}): ");
    stdout().flush().unwrap();

    let mut input_category = String::new();
    io::stdin().read_line(&mut input_category).unwrap();
    if !CATEGORIES.iter().any(|&s| s == input_category.trim()) {
        println!("Enter valid category (ex. {cats})...");
        thread::sleep(Duration::from_secs(3));
        return;
    }

    let (game_path, _lib) = get_game_path().unwrap();
    let full_path = game_path
        .join("garrysmod")
        .join("addons")
        .join(ADDON_NAME)
        .join("sound")
        .join("lanrp")
        .join("music")
        .join(&input_category.trim());

    let read_dir = match fs::read_dir(&full_path) {
        Ok(res) => res,
        Err(err) => {
            println!("Error occured when counting items in category dir: {err}");
            thread::sleep(Duration::from_secs(1));
            return;
        }
    };

    if read_dir.count() == 0 {
        println!("Category is empty...");
        thread::sleep(Duration::from_secs(1));
        return;
    }

    // Name

    println!("////////////////////////");

    let dir_iter = match fs::read_dir(full_path) {
        Ok(res) => res,
        Err(err) => {
            println!("Error occured when reading category dir: {err}");
            thread::sleep(Duration::from_secs(1));
            return;
        }
    };

    let mut iteration = 0;
    let mut sound_list = Vec::new();

    for entry in dir_iter {
        let entry = match entry {
            Ok(res) => res,
            Err(err) => {
                println!("Error occured when getting directory item: {err}");
                thread::sleep(Duration::from_secs(1));
                return;
            }
        };
        let path = entry.path();
        let file_name = path.file_name().unwrap().display().to_string();

        sound_list.push(file_name.to_string());

        println!("[{iteration}] {file_name}");

        iteration = iteration + 1;
    }

    println!("////////////////////////");

    print!("Choose file: ");
    let _ = stdout().flush();

    let mut input_num_str = String::new();
    io::stdin().read_line(&mut input_num_str).unwrap();

    let Ok(index) = input_num_str.trim().parse::<usize>() else {
        println!("Enter a valid number");
        thread::sleep(Duration::from_secs(1));
        return;
    };

    match sound_list.get(index) {
        Some(file_name) => {
            println!("Deleting: {}", file_name);

            let deleted = delete_music_api(file_name, input_category.trim());
            match deleted {
                Ok(()) => {
                    println!("File deleted!");
                    thread::sleep(Duration::from_secs(1));
                    return;
                }
                Err(err) => {
                    println!("{}: {err}", style("Error occured when deleting file").red());
                    thread::sleep(Duration::from_secs(3));
                    return;
                }
            }
        }
        None => {
            println!("File with this number not found!");
            thread::sleep(time::Duration::from_secs(1));
            return;
        }
    }
}

//
//
// Deleting music
//
//

fn list_music() {
    // Check

    if !is_addon_folder_exists() {
        println!("Addon folder not found. Create it in the menu...");
        thread::sleep(Duration::from_secs(3));
        return;
    }

    clear().unwrap();
    let _ = set_app_title("lanrp-music-rs - music list");

    let Some((game_path, _lib)) = get_game_path() else {
        return;
    };
    let addon_path = game_path
        .join("garrysmod")
        .join("addons")
        .join(ADDON_NAME)
        .join("sound")
        .join("lanrp")
        .join("music");

    for cat in CATEGORIES {
        let cat_path = addon_path.join(cat);
        let iter = match fs::read_dir(cat_path) {
            Ok(res) => res,
            Err(err) => {
                println!(
                    "{}: {err}",
                    style("Error occured when reading category directory").red()
                );
                thread::sleep(Duration::from_secs(3));
                return;
            }
        };
        println!("{}", cat);

        let mut i = 0;
        for file in iter {
            let file = match file {
                Ok(res) => res,
                Err(err) => {
                    println!(
                        "{}: {err}",
                        style("Error occured when reading directory file").red()
                    );
                    thread::sleep(Duration::from_secs(3));
                    return;
                }
            };
            println!(
                "  {}",
                style(file.file_name().display().to_string()).green()
            );

            i = i + 1
        }

        if i == 0 {
            println!("{}", style("  empty...").bright().black());
        }
    }

    println!("Press any key to return to menu...");

    io::stdout().flush().unwrap();

    let term = console::Term::stdout();
    let _ = term.read_key();
}

// Main-loop

fn main() {
    ctrlc::set_handler(move || {
        std::process::exit(0);
    })
    .expect("Error occured when setup Ctrl-C hook");

    let Some((app, _lib)) = get_game_path() else {
        println!("Garry's Mod not installed!");
        println!("Press any key to quit...");

        let _ = stdout().flush();

        let term = console::Term::stdout();
        let _ = term.read_key();

        return;
    };

    let app_path = app.display().to_string();

    let addon_path = app.join("garrysmod").join("addons").join(ADDON_NAME);

    loop {
        let addon_folder_exist = match fs::exists(&addon_path) {
            Ok(bool) => bool,
            Err(_) => false,
        };

        let folder_status = match addon_folder_exist {
            true => style("[Folder exists]").green().bold(),
            false => style("[Folder not exists]").red().bold(),
        };

        clear().unwrap();
        let _ = set_app_title("lanrp-music-rs");

        let mod_creds = style(CREDITS).bold();

        println!("---=== lanrp-music-rs ===---\n{mod_creds}\n\nGame Path: {app_path}\n");
        println!("1 - Create/Remove addon folder {}", folder_status);
        println!("2 - Add Music");
        println!("3 - Remove Music");
        println!("4 - List");
        println!("\n0 - Exit application");

        print!("\nEnter option: ");
        stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input_num: usize = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Enter a valid number");
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        match input_num {
            0 => break,
            1 => manage_addon_folders(),
            2 => add_music(),
            3 => delete_music(),
            4 => list_music(),
            _ => {
                println!("Unknown option, try again...");
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}
