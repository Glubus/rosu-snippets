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

#[derive(Clone, Debug)]
pub struct Snippets {
    pub hit_objects: Vec<HitObject>,
    pub timing_points: TimingPoint,
}

impl Snippets {
    pub fn new() -> Self {
        Self {
            hit_objects: Vec::new(),
            timing_points: TimingPoint::default(),
        }
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
        self.hit_objects = snippets.hit_objects.clone();
        self.timing_points = snippets.control_points.timing_points[0].clone();
        println!("Snippets loaded from file");
        Ok(())
    }

    pub fn normalize_hit_object(&self, hit_object: &mut HitObject, reference_time: f64) {
        hit_object.start_time -= reference_time;
    }

    pub fn save_snippets_to_file(&self, snippets_path: &str, reference_time: f64) -> Result<()> {
        println!("Saving snippets to file");
        let mut map = Beatmap::default();
        map.hit_objects = self.hit_objects.clone();
        for hit_object in map.hit_objects.iter_mut() {
            self.normalize_hit_object(hit_object, reference_time);
        }

        let mut t_points = self.timing_points.clone();
        t_points.time = 0.0;
        map.control_points.timing_points = vec![t_points];
        
        let snippets_path = std::path::Path::new("snippets").join(snippets_path);
        let file = File::create(snippets_path)?;
        let writer = BufWriter::new(file);
        map.encode(writer)?;
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
        self.collect_hit_objects(beatmap, snippets_maker)?;
        self.collect_timing_points(beatmap, snippets_maker)?;
        Ok(())
    }


    pub fn insert_snippets_to_beatmap(&self, process: &Process, state: &mut State) -> Result<()> {
        println!("Inserting snippets to beatmap");
        let beatmap_path = get_beatmap_path(process, state)?;
        let mut beatmap = Beatmap::from_path(&beatmap_path)?;
        let current_beat_len = if let Some(last_timing) = beatmap.control_points.timing_points.last() {
            last_timing.beat_len
        } else {
            600.0
        };

        let time_scale = current_beat_len / self.timing_points.beat_len;
        let placement_time = get_ig_time(process, state)?;

        for mut obj in self.hit_objects.clone() {
            obj.start_time = obj.start_time * time_scale + placement_time as f64;
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

#[derive(Clone, Debug)]
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

