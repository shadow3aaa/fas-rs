use std::collections::HashMap;

pub struct Weights {
    pub map: HashMap<i32, f64>,
}

impl Weights {
    pub fn weight(&self, cpus: &Vec<i32>) -> Option<f64> {
        if self.map.is_empty() {
            return None;
        }

        let mut weight = 1.0;
        for cpu in cpus {
            let partial_weight = *self.map.get(cpu)?;
            if partial_weight.is_normal() {
                weight += partial_weight;
            }
        }

        let weight = weight.min(1.5);

        Some(weight)
    }
}
