use flate2::read::GzDecoder;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::{fs, io};
use sigmars::{Event, MemBackend, SigmaCollection};

pub async fn process_events_from_dir(
    directory: &PathBuf,
    mut rules: SigmaCollection,
) -> Result<(), Box<dyn Error>>
{
    let mut backend = MemBackend::new().await;
    rules.init(&mut backend);
    let (_, file_paths, _) = count_files_recursive(directory)?;
    for path in file_paths {
        let log_contents = if path.ends_with("json") {
            fs::read_to_string(&path)?
        } else if path.ends_with("gz") {
            read_gz_file(&PathBuf::from(&path)).unwrap()
        } else {
            continue;
        };
        let json_value: Result<Value, _> = serde_json::from_str(&log_contents);
        match json_value {
            Ok(json_value) => {
                match json_value {
                    Value::Array(json_array) => {
                        for json_value in json_array {
                            let event: Event = Event::new(json_value.clone());
                            let res = rules.get_matches(&event).await;
                            if let Ok(res) = res {
                                if !res.is_empty() {
                                    println!("{:?}", res);
                                }
                            }
                        }
                    }
                    Value::Object(json_map) => {
                        if let Some(json_array) = json_map.get("Records") {
                            for json_value in json_array.as_array().unwrap() {
                                let event: Event = Event::new(json_value.clone());
                                let res = rules.get_matches(&event).await;
                                if let Ok(res) = res {
                                    if !res.is_empty() {
                                        println!("{:?}", res);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        // TODO: Handle unexpected JSON structure
                    }
                }
            }
            Err(_) => {
                // TODO: Handle unexpected JSON structure
            }
        }
    }
    Ok(())
}

fn count_files_recursive(directory: &PathBuf) -> Result<(usize, Vec<String>, u64), Box<dyn Error>> {
    let mut count = 0;
    let mut paths = Vec::new();
    let mut total_size = 0;
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "json" || ext == "gz" {
                    count += 1;
                    total_size += fs::metadata(&path)?.len();
                    paths.push(path.to_str().unwrap().to_string());
                }
            }
        } else if path.is_dir() {
            let (sub_count, sub_paths, sub_size) = count_files_recursive(&path)?;
            count += sub_count;
            total_size += sub_size;
            paths.extend(sub_paths);
        }
    }
    Ok((count, paths, total_size))
}

pub fn read_gz_file(file_path: &PathBuf) -> io::Result<String> {
    let file = File::open(file_path)?;
    let mut decoder = GzDecoder::new(BufReader::new(file));
    let mut contents = String::new();
    decoder.read_to_string(&mut contents)?;
    Ok(contents)
}
pub fn load_json_from_file(log_contents: &str) -> Result<Vec<Event>, Box<dyn Error>> {
    let mut events = Vec::new();
    let json_value: Value = serde_json::from_str(log_contents)?;
    match json_value {
        Value::Array(json_array) => {
            for json_value in json_array {
                let event: Event = Event::new(json_value.clone());
                events.push(event);
            }
        }
        Value::Object(json_map) => {
            if let Some(json_array) = json_map.get("Records") {
                for json_value in json_array.as_array().unwrap() {
                    let event: Event = Event::new(json_value.clone());
                    events.push(event);
                }
            }
        }
        _ => {
            eprintln!("Unexpected JSON structure in file:");
        }
    }
    Ok(events)
}

pub fn get_content(f: &PathBuf) -> String {
    let path = f.display().to_string();
    if path.ends_with(".json") {
        fs::read_to_string(f).unwrap_or_default()
    } else if path.ends_with(".gz") {
        read_gz_file(f).unwrap_or_default()
    } else {
        "".to_string()
    }
}

