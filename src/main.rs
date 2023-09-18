use std::{
    env,
    fs,
    path::Path,
    collections::HashMap,
    time::Duration,
};
use notify::{
    RecursiveMode,
    Watcher,
    Result,
};
use notify_debouncer_full::{
    new_debouncer, 
    Debouncer,
    FileIdMap,
};
use serde::Deserialize;

/// debouncer <asjidkalam, 2023>

//// OBJECTIVES
/// 1. cache the files on memory in startup                 [DONE]
/// 2. check for events on restricted file                  [DONE]
/// 3. restore backup files for modify/delete events        [DONE]
/// 5. cache files again if changed, from INotifyWatcher    [DONE]
/// 4. setup systemd                                        [TODO]

#[derive(Debug, Deserialize)]
struct Config {
    watched_paths: Vec<String>,
}

fn load_config(filename: &str) -> Result<Config> {
    let config_file = fs::read_to_string(filename)?;
    let config = serde_json::from_str(&config_file).map_err(|e| {
        notify::Error::io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    })?;
    Ok(config)
}

fn cache_files(path: &str, deb: &mut Debouncer<notify::INotifyWatcher, FileIdMap>) {
    if let Err(e) = deb.watcher().watch(Path::new(path), RecursiveMode::Recursive) {
        eprintln!("Failed to watch path: {}. Error: {:?}", path, e);
    }
    deb.cache().add_root(Path::new(path), RecursiveMode::Recursive); 
}

fn initialize_cache(path: &str, store: &mut HashMap<String, String>, debouncer: &mut Debouncer<notify::INotifyWatcher, FileIdMap>) {
    println!("[CACHE]: {}", path);
    
    if let Ok(org_file) = fs::read_to_string(path) {
        store.insert(path.to_string(), org_file);
    } else {
        eprintln!("Unable to read files to restrict: {}", path);
    }

    cache_files(path, debouncer);
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <config_file_path>", args[0]);
        return Ok(());
    }

    let config = load_config(&args[1])?;
    let mut store: HashMap<String, String> = HashMap::new();
    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_secs(1), None, tx)?;

    for path in &config.watched_paths {
        initialize_cache(path, &mut store, &mut debouncer);
    }

    for result in rx {
        match result {
            Ok(events) => {
                for event in events {
                    let event_path_result = event.paths.get(0).and_then(|path| path.clone().into_os_string().into_string().ok());
                    
                    if let Some(event_path) = event_path_result {                        
                        if let Some(cached_content) = store.get(&event_path) {
                            if let Err(err) = fs::write(&event_path, cached_content) {
                                eprintln!("Unable to write file: {:?}", err);
                            }
                        }

                        println!("[CACHE] File restored! - {}", event_path);
                        // after the file is restored, cache again.
                        cache_files(&event_path, &mut debouncer);
                    } else {
                        eprintln!("Event path not found or not valid UTF-8.");
                    }
                }
            }
            Err(errors) => {
                for error in errors {
                    eprintln!("{:?}", error);
                }
            }
        }
    }

    Ok(())
}
