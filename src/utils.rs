use rosu_map::section::hit_objects::{HitObject, HitObjectKind};


pub fn hit_object_to_string(hit_object: HitObject) -> String {
    match hit_object.kind {
        HitObjectKind::Circle(circle) => {
            format!("{},{},{},{},0,0:0:0:0:0:", circle.pos.x, circle.pos.y, hit_object.start_time, 1<<0)
        }
        HitObjectKind::Slider(slider) => {
            format!("{},{},{},{},0,B|0:0,1,100,0:0:0:0", slider.pos.x, slider.pos.y, hit_object.start_time, 1<<1)
        }
        HitObjectKind::Spinner(spinner) => {
            format!("256,192,{},{},0,{},0:0:0:0:", hit_object.start_time, 1<<3, hit_object.start_time + 1000.0)
        }
        HitObjectKind::Hold(hold) => {
            format!("{},0,{},{},0,{},0:0:0:0:", hold.pos_x, hit_object.start_time, 1<<7, hit_object.start_time + hold.duration) 
        }
        _ => String::new()
    }
}