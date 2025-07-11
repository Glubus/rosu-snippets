use eyre::Result;
use rosu_mem::process::{Process};
use rosu_memory_lib::reader::structs::{State};
use rosu_memory_lib::reader::gameplay::stable::memory::get_ig_time;
use rosu_map::section::timing_points::{TimingPoint};
use rosu_map::section::hit_objects::{HitObject};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use rosu_map::{Beatmap, DecodeBeatmap};
use rosu_memory_lib::reader::beatmap::stable::file::get_beatmap_path;
use rosu_map::section::hit_objects::HitObjectKind;
use rand::seq::SliceRandom;
use rand::Rng;
#[derive(Clone, Debug)]
pub struct Snippets {
    pub name: String,
    pub hit_objects: Vec<HitObject>,
    pub timing_points: TimingPoint,
    pub is_saved: bool,
    pub should_shuffle: bool,  // Nouvelle option pour le shuffle
    pub keycount: usize, // mania only 
    pub tags: Vec<String>,
}

impl Snippets {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            hit_objects: Vec::new(),
            timing_points: TimingPoint::default(),
            is_saved: false,
            should_shuffle: false,
            keycount: 4,
            tags: Vec::new(),
        }
    }

    /// TODO: Supporter plus que 4 colonnes en récupérant le nombre de colonnes depuis la beatmap
    fn shuffle_column(&self, column: usize) -> usize {
        // Pour l'instant on hardcode 4 colonnes
        let mut columns: Vec<usize> = (0..self.keycount).collect();
        columns.shuffle(&mut rand::thread_rng());
        columns[column]
    }

    pub fn load_snippets_memory_path(&mut self, process: &Process, state: &mut State, snippets_maker: &SnippetsMaker) -> Result<()> {
        println!("Loading snippets from memory path");
        let beatmap_path = get_beatmap_path(process, state)?;
        let beatmap = Beatmap::from_path(&beatmap_path)?;
        println!("Beatmap loaded from memory path: {beatmap_path}");
        self.load_snippets_from_beatmap(&beatmap, snippets_maker)
    }

    pub fn load_snippets(&mut self, snippets_path: &str) -> Result<()> {
        println!("Loading snippets from file");
        let snippets_path = std::path::Path::new("snippets").join(snippets_path);
        let file = File::open(snippets_path)?;
        let reader = BufReader::new(file);

        // Decode snippets
        let snippets = Beatmap::decode(reader)?;
        self.name = snippets.title.clone();
        self.hit_objects = snippets.hit_objects.clone();
        self.timing_points = snippets.control_points.timing_points[0].clone();
        self.tags = snippets.tags.split(" ").map(|s| s.to_string()).collect();
        self.keycount = snippets.circle_size as usize;
        self.is_saved = true;
        println!("Snippets loaded from file");
        Ok(())
    }


    pub fn save_snippets_to_file(&mut self, snippets_name: &str) -> Result<()> {
        println!("Saving snippets to file");
        self.name = snippets_name.to_string();  // Mettre à jour le nom avec le nom du fichier

        let mut map = Beatmap::default();
        map.title = self.name.clone();
        map.hit_objects = self.hit_objects.clone();
        map.tags = self.tags.clone().join(" ");
        map.circle_size = self.keycount as f32;
        let mut t_points = self.timing_points.clone();
        t_points.time = 0.0;
        map.control_points.timing_points = vec![t_points];
        
        let snippets_path = std::path::Path::new("snippets").join(snippets_name);
        let file = File::create(snippets_path)?;
        let writer = BufWriter::new(file);
        map.encode(writer)?;
        self.is_saved = true;
        println!("Snippets saved to file");
        Ok(())
    }

    pub fn collect_hit_objects(&mut self, beatmap: &Beatmap, snippets_maker: &SnippetsMaker) -> Result<()> {
        for hit_object in beatmap.hit_objects.clone() {
            if hit_object.start_time >= snippets_maker.time_start as f64 && hit_object.start_time <= snippets_maker.time_end as f64 {
                self.hit_objects.push(hit_object);
            }
        }
        Ok(())
    }

    pub fn collect_timing_points(&mut self, beatmap: &Beatmap, snippets_maker: &SnippetsMaker) -> Result<()> {
        let mut closest_point = None;
        let mut min_distance = f64::MAX;
    
        for timing_point in beatmap.control_points.timing_points.clone() {
            if timing_point.time >= snippets_maker.time_start as f64 && timing_point.time <= snippets_maker.time_end as f64 {
                self.timing_points = timing_point;
                return Ok(());
            }

            let distance = (timing_point.time - snippets_maker.time_start as f64).abs();
            if distance < min_distance {
                min_distance = distance;
                closest_point = Some(timing_point);
            }
        }
        if closest_point.is_some() {
            self.timing_points = closest_point.unwrap();
        }
        Ok(())
    }
    
    pub fn load_snippets_from_beatmap(&mut self, beatmap: &Beatmap, snippets_maker: &SnippetsMaker) -> Result<()> {
        println!("Loading snippets from beatmap");
        self.collect_hit_objects(beatmap, snippets_maker)?;
        self.collect_timing_points(beatmap, snippets_maker)?;
        self.keycount = beatmap.circle_size as usize;
        
        // Normaliser les temps des notes par rapport au temps de début
        for hit_object in self.hit_objects.iter_mut() {
            hit_object.start_time -= snippets_maker.time_start as f64;
        }
        
        println!("Snippets loaded from beatmap");
        Ok(())
    }


    pub fn insert_snippets_to_beatmap(&self, process: &Process, state: &mut State) -> Result<()> {
        println!("Inserting snippets to beatmap");
        let beatmap_path = get_beatmap_path(process, state)?;
        let mut beatmap = Beatmap::from_path(&beatmap_path)?;
        beatmap.tags = self.tags.clone().join(" ");
        let current_beat_len = if let Some(last_timing) = beatmap.control_points.timing_points.last() {
            last_timing.beat_len
        } else {
            600.0
        };

        let time_scale = current_beat_len / self.timing_points.beat_len;
        let placement_time = get_ig_time(process, state)?;

        // Générer le mapping des colonnes une seule fois si shuffle est activé
        let column_mapping: Option<Vec<usize>> = if self.should_shuffle {
            let mut columns: Vec<usize> = (0..self.keycount).collect();
            columns.shuffle(&mut rand::thread_rng());
            Some(columns)
        } else {
            None
        };

        for mut obj in self.hit_objects.clone() {
            obj.start_time = obj.start_time * time_scale + placement_time as f64;
            
            // Appliquer le shuffle si activé
            if self.should_shuffle {
                if let Some(mapping) = &column_mapping {
                    match obj.kind {
                        HitObjectKind::Circle(ref mut h) => {
                            let column = (h.pos.x / 512.0 * self.keycount as f32) as usize % self.keycount;
                            let new_column = mapping[column];
                            h.pos.x = (new_column as f32 * 512.0 / self.keycount as f32) + (512.0 / 8.0); // Centrer dans la colonne
                        }
                        HitObjectKind::Hold(ref mut h) => {
                            let column = (h.pos_x / 512.0 * self.keycount as f32) as usize % self.keycount;
                            let new_column = mapping[column];
                            h.pos_x = (new_column as f32 * 512.0 / self.keycount as f32) + (512.0 / 8.0); // Centrer dans la colonne
                        }
                        _ => continue,
                    }
                }
            }

            beatmap.hit_objects.push(obj);
        }

        beatmap.hit_objects.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());
        beatmap.encode_to_path(&beatmap_path)?;
        println!("Snippets inserted to beatmap");
        Ok(())
    }
}


#[derive(Clone, Debug)]
pub struct SnippetsMaker {
    pub time_start: i32,
    pub time_end: i32,
    pub next_update: NextUpdate,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NextUpdate {
    TimeStart,
    TimeEnd,
}

impl SnippetsMaker {
    pub fn new() -> Self {
        Self {
            time_start: 0,
            time_end: 0,
            next_update: NextUpdate::TimeStart,
        }
    }

    pub fn set_ig_time(&mut self, ig_time: i32) -> Result<(NextUpdate), eyre::Error> {

        match self.next_update {
            NextUpdate::TimeStart => {
                println!("Time start: {}", ig_time);
                self.time_start = ig_time;
                self.next_update = NextUpdate::TimeEnd;
                Ok(NextUpdate::TimeEnd)
            }
            NextUpdate::TimeEnd => {
                println!("Time end: {}", ig_time);
                self.time_end = ig_time;
                self.next_update = NextUpdate::TimeStart;
                Ok(NextUpdate::TimeStart)
            }
        }
    }

    pub fn set_next(&mut self, p: &Process, state: &mut State) -> Result<(NextUpdate), eyre::Error> 
    {
        if let Ok(ig_time) = get_ig_time(p, state) {
            self.set_ig_time(ig_time)
        } else {
            Err(eyre::eyre!("Failed to get ig time"))
        }
    }
}
