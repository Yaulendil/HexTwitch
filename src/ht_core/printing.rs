use std::collections::HashMap;


pub struct States {
    map: Option<HashMap<String, String>>,
}


impl States {
    fn ensure(&mut self) {
        match &self.map {
            None => {
                self.map.replace(HashMap::new()).unwrap();
            }
            Some(map) => {}
        }
    }

    pub fn get(&mut self, channel: &str) -> &str {
        self.ensure();

        match self.map.as_ref().unwrap().get(channel) {
            None => "",
            Some(val) => val,
        }
    }

    pub fn set(&mut self, channel: String, state: &str) {
        self.ensure();

        let mut new = state.to_string();
        self.map.as_mut().unwrap().get_mut(&channel).replace(&mut new);
    }
}


pub static mut USERSTATE: States = States {
    map: None,
};
