use super::file;

use std::collections::HashMap;

pub struct Dupes {
    names: HashMap<String, i32>,
}

impl Dupes {
    pub fn new() -> Dupes {
        Dupes {
            names: HashMap::new(),
        }
    }

    pub fn use_name(&mut self, name: &str) -> (String, bool) {
        let fname = name.to_string();
        match self.names.get(&fname) {
            Some(count) => {
                let next_name = file::splice_name(name, count);
                let next_count = count + 1;
                self.names.insert(fname.clone(), next_count);
                (next_name, true)
            }
            None => {
                self.names.insert(fname.clone(), 1);
                (fname, false)
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn unit_dupes_add() {
        let mut dupes = Dupes::new();
        assert_eq!(dupes.use_name("test.png"), ("test.png".into(), false));
        assert_eq!(dupes.use_name("test.png"), ("test_1.png".into(), true));
        assert_eq!(dupes.use_name("test.png"), ("test_2.png".into(), true));
        assert_eq!(dupes.use_name("test.png"), ("test_3.png".into(), true));
        assert_eq!(dupes.use_name("test.jpg"), ("test.jpg".into(), false));
    }
}
