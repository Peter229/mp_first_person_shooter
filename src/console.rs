use std::collections::HashMap;

pub struct Console {
    log: String,
    timing: HashMap<String, f32>,
}

impl Console {

    pub fn new() -> Self {

        Self { log: "".to_string(), timing: HashMap::new() }
    }

    pub fn output_to_console(&mut self, output: &str) {

        self.log += &(output.to_string() + "\n");
    }

    pub fn get_log(&self) -> &String {

        &self.log
    }

    pub fn insert_timing(&mut self, name: &str, time: f32) {

        self.timing.insert(name.to_string(), time);
    }

    pub fn get_timings_string(&self) -> String {

        let mut string = "".to_string();

        for (name, time) in &self.timing {
            string += &format!("{} took {}ms\n", name, time);
        }

        string
    }
}