mod utils;
mod snippets;
use crate::snippets::structs::{SnippetsMaker, Snippets};
use rosu_memory_lib::{init_loop};
use rdev::{listen, Event, EventType, Key};
use std::sync::{Arc, Mutex};

struct AppState {
        snippets: Snippets,
        snippets_maker: SnippetsMaker,
}





// fn get_time_points(b: &Beatmap, timing_points: &SnippetsTimingPoints) -> TimingPoint {
//     let mut closest_point = None;
//     let mut min_distance = f64::MAX;

//     for timing_point in b.control_points.timing_points.clone() {
//         // Si on trouve un point dans l'intervalle, on le retourne directement
//         if timing_point.time >= timing_points.time_start as f64 && timing_point.time <= timing_points.time_end as f64 {
//             return timing_point;
//         }
        
//         // Sinon on garde trace du point le plus proche de time1
//         let distance = (timing_point.time - timing_points.time_start as f64).abs();
//         if distance < min_distance {
//             min_distance = distance;
//             closest_point = Some(timing_point);
//         }
//     }
    
//     // Retourne le point le plus proche (il y a forcément au moins un point)
//     closest_point.unwrap()
// }
// fn save_snippets(process: &Process, state: &mut State, app_state: Arc<Mutex<AppState>>) -> Result<()> {
//     println!("Saving snippets");
//     let mut app = app_state.lock().unwrap();
//     let timing_points = app.timing_points.clone();
//     let beatmap_file = get_beatmap_path(process, state)?;
//     let b = Beatmap::from_path(beatmap_file)?;
//     let hit_objects = b.hit_objects.clone();
//     let mut snippets = Vec::new();
    
//     // Collect hit objects in the time range
//     for hit_object in hit_objects {
//         if hit_object.start_time >= timing_points.time_start as f64 && hit_object.start_time <= timing_points.time_end as f64 {
//             snippets.push(hit_object);  
//         }
//     }

//     // Sort snippets by time and get the first time
//     snippets.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
//     let first_time = if let Some(first) = snippets.first() {
//         first.start_time
//     } else {
//         return Ok(()) // No hit objects found
//     };

//     println!("Starting to create file");
//     fs::create_dir_all("snippets")?;
//     let file_path = std::path::Path::new("snippets").join(format!("{}.snippets", "test"));
//     let mut file = std::fs::File::create(file_path)?;
//     println!("File created");


//     // Get and normalize timing point
//     println!("Getting time points");
//     let point = get_time_points(&b, &timing_points);
//     println!("Writing timing point");
//     file.write_all(format!("[TimingPoints]\n").as_bytes())?;
//     // Normalize timing point time relative to first hit object
//     file.write_all(format!("{},{},{},{}\n", 0, point.beat_len, 0, 0).as_bytes())?;
    
//     // Write normalized hit objects
//     file.write_all(format!("[HitObjects]\n").as_bytes())?;
//     for snippet in snippets {
//         let normalized_hit_object = normalize_hit_object(snippet, first_time);
//         file.write_all(format!("{}\n", hit_object_to_string(normalized_hit_object)).as_bytes())?;
//     }
//     println!("File written");
//     Ok(())
// }

// fn normalize_hit_object(hit_object: HitObject, reference_time: f64) -> HitObject {
//     let mut normalized = hit_object;
//     normalized.start_time -= reference_time;
//     normalized
// }



fn main() -> eyre::Result<()> {
    let (mut state, process) = init_loop(500)?;
    println!("Successfully initialized!");
    
    let app_state = Arc::new(Mutex::new(AppState {
        snippets: Snippets::new(),
        snippets_maker: SnippetsMaker::new(),
    }));
    
    let app_state_clone = Arc::clone(&app_state);
    let callback = move |event: Event| {
        match event.event_type {
            EventType::KeyPress(Key::Num1) => {
                let mut app = app_state_clone.lock().unwrap();
                app.snippets_maker.set_next(&process, &mut state);
            },
            EventType::KeyPress(Key::Num2) => {
                let mut app = app_state_clone.lock().unwrap();
                let snippets_maker = app.snippets_maker.clone();
                app.snippets.load_snippets_memory_path(&process, &mut state, &snippets_maker);
            },
            EventType::KeyPress(Key::Num3) => {
                let mut app = app_state_clone.lock().unwrap();
                app.snippets.save_snippets_to_file("test.snippets", app.snippets_maker.time_start as f64);
            },
            EventType::KeyPress(Key::Num4) => {
                let mut app = app_state_clone.lock().unwrap();
                app.snippets.load_snippets("test.snippets");
            },
            EventType::KeyPress(Key::Num5) => {
                let mut app = app_state_clone.lock().unwrap();
                app.snippets.insert_snippets_to_beatmap(&process, &mut state);
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