mod utils;
mod snippets;
use crate::utils::hit_object_to_string;
use crate::snippets::structs::SnippetsTimingPoints;
use rosu_memory_lib::reader::gameplay::stable::memory::get_ig_time;
use rosu_memory_lib::{init_loop};
use rosu_mem::process::{Process, ProcessTraits};
use rosu_memory_lib::reader::structs::{State};

use eyre::Result;
use rdev::{listen, Event, EventType, Key};
use std::sync::{Arc, Mutex};
use rosu_memory_lib::reader::beatmap::stable::file::get_beatmap_path;
use rosu_map::{Beatmap, DecodeBeatmap};
use rosu_map::section::hit_objects::{HitObject, HitObjectKind};
use std::io::Write;
use rosu_map::section::timing_points::{TimingPoint, TimingPoints};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use rosu_map::section::hit_objects::HitObjects;


struct AppState {
        timing_points: SnippetsTimingPoints,
}



fn set_timing_points(process: &Process, state: &mut State, app_state: Arc<Mutex<AppState>>) -> Result<()> {
    if let Ok(ig_time) = get_ig_time(process, state) {
        let mut app = app_state.lock().unwrap();
        app.timing_points.set_next(ig_time)?;
        Ok(())
    } else {
        Err(eyre::eyre!("Failed to get ig time"))
    }
}


fn get_time_points(b: &Beatmap, timing_points: &SnippetsTimingPoints) -> TimingPoint {
    let mut closest_point = None;
    let mut min_distance = f64::MAX;

    for timing_point in b.control_points.timing_points.clone() {
        // Si on trouve un point dans l'intervalle, on le retourne directement
        if timing_point.time >= timing_points.time_start as f64 && timing_point.time <= timing_points.time_end as f64 {
            return timing_point;
        }
        
        // Sinon on garde trace du point le plus proche de time1
        let distance = (timing_point.time - timing_points.time_start as f64).abs();
        if distance < min_distance {
            min_distance = distance;
            closest_point = Some(timing_point);
        }
    }
    
    // Retourne le point le plus proche (il y a forcément au moins un point)
    closest_point.unwrap()
}
fn save_snippets(process: &Process, state: &mut State, app_state: Arc<Mutex<AppState>>) -> Result<()> {
    println!("Saving snippets");
    let mut app = app_state.lock().unwrap();
    let timing_points = app.timing_points.clone();
    let beatmap_file = get_beatmap_path(process, state)?;
    let b = Beatmap::from_path(beatmap_file)?;
    let hit_objects = b.hit_objects.clone();
    let mut snippets = Vec::new();
    
    // Collect hit objects in the time range
    for hit_object in hit_objects {
        if hit_object.start_time >= timing_points.time_start as f64 && hit_object.start_time <= timing_points.time_end as f64 {
            snippets.push(hit_object);  
        }
    }

    // Sort snippets by time and get the first time
    snippets.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
    let first_time = if let Some(first) = snippets.first() {
        first.start_time
    } else {
        return Ok(()) // No hit objects found
    };

    println!("Starting to create file");
    fs::create_dir_all("snippets")?;
    let file_path = std::path::Path::new("snippets").join(format!("{}.snippets", "test"));
    let mut file = std::fs::File::create(file_path)?;
    println!("File created");


    // Get and normalize timing point
    println!("Getting time points");
    let point = get_time_points(&b, &timing_points);
    println!("Writing timing point");
    file.write_all(format!("[TimingPoints]\n").as_bytes())?;
    // Normalize timing point time relative to first hit object
    file.write_all(format!("{},{},{},{}\n", 0, point.beat_len, 0, 0).as_bytes())?;
    
    // Write normalized hit objects
    file.write_all(format!("[HitObjects]\n").as_bytes())?;
    for snippet in snippets {
        let normalized_hit_object = normalize_hit_object(snippet, first_time);
        file.write_all(format!("{}\n", hit_object_to_string(normalized_hit_object)).as_bytes())?;
    }
    println!("File written");
    Ok(())
}

fn normalize_hit_object(hit_object: HitObject, reference_time: f64) -> HitObject {
    let mut normalized = hit_object;
    normalized.start_time -= reference_time;
    normalized
}

fn load_snippets(process: &Process, state: &mut State) -> Result<()> {
    println!("Loading snippets");
    
    // Get current beatmap path and load it
    let beatmap_file = get_beatmap_path(process, state)?;
    let mut beatmap = Beatmap::from_path(&beatmap_file)?;
    
    // Load snippets file
    let snippets_path = std::path::Path::new("snippets").join("test.snippets");
    let file = File::open(snippets_path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        println!("Line: {:?}", line);
    }
    let snippets_path = std::path::Path::new("snippets").join("test.snippets");
    let file = File::open(snippets_path)?;
    let reader = BufReader::new(file);
    // Decode snippets
    let snippets = HitObjects::decode(reader)?;
    
    println!("Snippets: {:?}", snippets.hit_objects.len());
    // Get current beatmap's last timing point beat_len
    let current_beat_len = if let Some(last_timing) = beatmap.control_points.timing_points.last() {
        last_timing.beat_len
    } else {
        600.0 // Default BPM 100
    };

    let snippets_path = std::path::Path::new("snippets").join("test.snippets");
    let file = File::open(snippets_path)?;
    let reader = BufReader::new(file);
    let test = TimingPoints::decode(reader)?;

    // Calculate time scaling factor
    let time_scale = current_beat_len / test.control_points.timing_points[0].beat_len;
    
    // Get last object time in beatmap
    let placement_time = get_ig_time(process, state)?;
    
    // Add offset and scale times for each hit object
    for mut obj in snippets.hit_objects {
        obj.start_time = obj.start_time * time_scale + placement_time as f64; // 1s gap
        beatmap.hit_objects.push(obj);
    }
    
    // Sort hit objects by time
    beatmap.hit_objects.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
    
    // Write the modified beatmap back to file
    beatmap.encode_to_path(&beatmap_file)?;
    println!("Snippets loaded and saved to beatmap");
    Ok(())
}

fn main() -> Result<()> {
    let (mut state, process) = init_loop(500)?;
    println!("Successfully initialized!");
    
    let app_state = Arc::new(Mutex::new(AppState {
        timing_points: SnippetsTimingPoints::new()
    }));
    
    let app_state_clone = Arc::clone(&app_state);
    let callback = move |event: Event| {
        match event.event_type {
            EventType::KeyPress(Key::Num2) => {
                set_timing_points(&process, &mut state, Arc::clone(&app_state_clone));
            },
            EventType::KeyPress(Key::Num3) => {
                save_snippets(&process, &mut state, Arc::clone(&app_state_clone));
            },
            EventType::KeyPress(Key::Num4) => {
                if let Err(e) = load_snippets(&process, &mut state) {
                    println!("Error loading snippets: {}", e);
                }
            },
            _ => (),
        }
    };

    // Start listening to keyboard events
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error)
    }
    
    Ok(())
}