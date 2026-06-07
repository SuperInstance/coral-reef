//! Ecosystem simulation for diverse agent populations.
//!
//! Models coral reef ecosystems with individual agents (polyps), colonies,
//! symbiotic relationships, and stress response/recovery mechanisms.


// ============================================================================
// polyp module
// ============================================================================

pub mod polyp {
    use std::collections::HashMap;

    /// An individual agent (polyp) in the ecosystem.
    #[derive(Debug, Clone)]
    pub struct Polyp {
        pub id: u64,
        pub health: f64,        // 0.0 - 1.0
        pub energy: f64,        // 0.0 - 1.0
        pub age: u32,
        pub species: String,
        pub position: (f64, f64),
        pub traits: HashMap<String, f64>,
    }

    impl Polyp {
        pub fn new(id: u64, species: &str, position: (f64, f64)) -> Self {
            Self {
                id,
                health: 1.0,
                energy: 0.8,
                age: 0,
                species: species.to_string(),
                position,
                traits: HashMap::new(),
            }
        }

        pub fn with_health(mut self, health: f64) -> Self {
            self.health = health.clamp(0.0, 1.0);
            self
        }

        pub fn with_energy(mut self, energy: f64) -> Self {
            self.energy = energy.clamp(0.0, 1.0);
            self
        }

        pub fn set_trait(&mut self, name: &str, value: f64) {
            self.traits.insert(name.to_string(), value);
        }

        pub fn get_trait(&self, name: &str) -> f64 {
            self.traits.get(name).copied().unwrap_or(0.0)
        }

        pub fn tick(&mut self) {
            self.age += 1;
            self.energy = (self.energy - 0.01).max(0.0);
            if self.energy < 0.1 {
                self.health = (self.health - 0.02).max(0.0);
            }
            // Aging effect
            if self.age > 1000 {
                self.health = (self.health - 0.005).max(0.0);
            }
        }

        pub fn feed(&mut self, amount: f64) {
            self.energy = (self.energy + amount).min(1.0);
        }

        pub fn heal(&mut self, amount: f64) {
            self.health = (self.health + amount).min(1.0);
        }

        pub fn damage(&mut self, amount: f64) {
            self.health = (self.health - amount).max(0.0);
        }

        pub fn is_alive(&self) -> bool {
            self.health > 0.0
        }

        pub fn distance_to(&self, other: &Polyp) -> f64 {
            let dx = self.position.0 - other.position.0;
            let dy = self.position.1 - other.position.1;
            (dx * dx + dy * dy).sqrt()
        }

        pub fn move_towards(&mut self, target: (f64, f64), speed: f64) {
            let dx = target.0 - self.position.0;
            let dy = target.1 - self.position.1;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 0.001 {
                self.position.0 += dx / dist * speed;
                self.position.1 += dy / dist * speed;
            }
        }

        pub fn fitness(&self) -> f64 {
            self.health * 0.5 + self.energy * 0.3 + (1.0 - (self.age as f64 / 2000.0).min(1.0)) * 0.2
        }

        pub fn can_reproduce(&self) -> bool {
            self.health > 0.6 && self.energy > 0.5 && self.age > 50
        }

        pub fn reproduce(&self, new_id: u64) -> Polyp {
            let offset_x = (self.id as f64 * 0.1).sin() * 0.5;
            let offset_y = (self.id as f64 * 0.2).cos() * 0.5;
            Polyp::new(new_id, &self.species, (self.position.0 + offset_x, self.position.1 + offset_y))
                .with_energy(self.energy * 0.5)
        }
    }
}

// ============================================================================
// colony module
// ============================================================================

pub mod colony {
    use super::polyp::Polyp;
    use std::collections::HashMap;

    /// A colony (cluster) of agents.
    #[derive(Debug, Clone)]
    pub struct Colony {
        pub id: String,
        pub polyps: HashMap<u64, Polyp>,
        pub species: String,
        pub cohesion: f64, // 0.0 - 1.0, how tightly bound
    }

    impl Colony {
        pub fn new(id: &str, species: &str) -> Self {
            Self { id: id.to_string(), polyps: HashMap::new(), species: species.to_string(), cohesion: 0.8 }
        }

        pub fn add(&mut self, polyp: Polyp) {
            self.polyps.insert(polyp.id, polyp);
        }

        pub fn remove(&mut self, id: u64) -> Option<Polyp> {
            self.polyps.remove(&id)
        }

        pub fn get(&self, id: u64) -> Option<&Polyp> {
            self.polyps.get(&id)
        }

        pub fn get_mut(&mut self, id: u64) -> Option<&mut Polyp> {
            self.polyps.get_mut(&id)
        }

        pub fn population(&self) -> usize {
            self.polyps.len()
        }

        pub fn is_empty(&self) -> bool {
            self.polyps.is_empty()
        }

        pub fn average_health(&self) -> f64 {
            if self.polyps.is_empty() { return 0.0; }
            self.polyps.values().map(|p| p.health).sum::<f64>() / self.polyps.len() as f64
        }

        pub fn average_energy(&self) -> f64 {
            if self.polyps.is_empty() { return 0.0; }
            self.polyps.values().map(|p| p.energy).sum::<f64>() / self.polyps.len() as f64
        }

        pub fn alive_count(&self) -> usize {
            self.polyps.values().filter(|p| p.is_alive()).count()
        }

        pub fn remove_dead(&mut self) -> usize {
            let before = self.polyps.len();
            self.polyps.retain(|_, p| p.is_alive());
            before - self.polyps.len()
        }

        pub fn tick_all(&mut self) {
            for polyp in self.polyps.values_mut() {
                polyp.tick();
            }
            self.update_cohesion();
        }

        pub fn feed_all(&mut self, amount: f64) {
            for polyp in self.polyps.values_mut() {
                polyp.feed(amount);
            }
        }

        pub fn center_of_mass(&self) -> (f64, f64) {
            if self.polyps.is_empty() { return (0.0, 0.0); }
            let n = self.polyps.len() as f64;
            let x: f64 = self.polyps.values().map(|p| p.position.0).sum::<f64>() / n;
            let y: f64 = self.polyps.values().map(|p| p.position.1).sum::<f64>() / n;
            (x, y)
        }

        pub fn update_cohesion(&mut self) {
            if self.polyps.len() < 2 { self.cohesion = 1.0; return; }
            let center = self.center_of_mass();
            let avg_dist: f64 = self.polyps.values()
                .map(|p| {
                    let dx = p.position.0 - center.0;
                    let dy = p.position.1 - center.1;
                    (dx * dx + dy * dy).sqrt()
                })
                .sum::<f64>() / self.polyps.len() as f64;
            self.cohesion = 1.0 / (1.0 + avg_dist);
        }

        pub fn total_fitness(&self) -> f64 {
            self.polyps.values().map(|p| p.fitness()).sum()
        }

        pub fn reproduction_candidates(&self) -> Vec<u64> {
            self.polyps.values().filter(|p| p.can_reproduce()).map(|p| p.id).collect()
        }

        pub fn polyp_ids(&self) -> Vec<u64> {
            self.polyps.keys().copied().collect()
        }
    }
}

// ============================================================================
// reef module
// ============================================================================

pub mod reef {
    use super::colony::Colony;
    
    use std::collections::HashMap;

    /// Environmental conditions for the reef.
    #[derive(Debug, Clone)]
    pub struct Environment {
        pub temperature: f64,
        pub light: f64,
        pub nutrients: f64,
        pub ph: f64,
    }

    impl Environment {
        pub fn tropical() -> Self {
            Self { temperature: 26.0, light: 0.8, nutrients: 0.6, ph: 8.1 }
        }

        pub fn optimal() -> Self {
            Self { temperature: 25.0, light: 0.9, nutrients: 0.7, ph: 8.2 }
        }

        pub fn stress_level(&self) -> f64 {
            let temp_stress = (self.temperature - 25.0).abs() / 10.0;
            let light_stress = (self.light - 0.8).abs();
            let nutrient_stress = (self.nutrients - 0.7).abs();
            let ph_stress = (self.ph - 8.2).abs() / 2.0;
            (temp_stress + light_stress + nutrient_stress + ph_stress) / 4.0
        }

        pub fn is_healthy(&self) -> bool {
            self.stress_level() < 0.2
        }
    }

    /// The full reef ecosystem.
    #[derive(Debug)]
    pub struct Reef {
        pub colonies: HashMap<String, Colony>,
        pub environment: Environment,
        pub tick_count: u64,
        next_polyp_id: u64,
    }

    impl Reef {
        pub fn new(environment: Environment) -> Self {
            Self { colonies: HashMap::new(), environment, tick_count: 0, next_polyp_id: 1 }
        }

        pub fn add_colony(&mut self, colony: Colony) {
            self.colonies.insert(colony.id.clone(), colony);
        }

        pub fn remove_colony(&mut self, id: &str) -> Option<Colony> {
            self.colonies.remove(id)
        }

        pub fn colony(&self, id: &str) -> Option<&Colony> {
            self.colonies.get(id)
        }

        pub fn colony_mut(&mut self, id: &str) -> Option<&mut Colony> {
            self.colonies.get_mut(id)
        }

        pub fn colony_count(&self) -> usize {
            self.colonies.len()
        }

        pub fn total_population(&self) -> usize {
            self.colonies.values().map(|c| c.population()).sum()
        }

        pub fn tick(&mut self) {
            self.tick_count += 1;
            for colony in self.colonies.values_mut() {
                colony.tick_all();
                // Apply environmental effects
                let stress = self.environment.stress_level();
                if stress > 0.3 {
                    for polyp in colony.polyps.values_mut() {
                        polyp.damage(stress * 0.01);
                    }
                }
                if self.environment.nutrients > 0.5 {
                    colony.feed_all(0.02 * self.environment.nutrients);
                }
                colony.remove_dead();
            }
        }

        pub fn allocate_polyp_id(&mut self) -> u64 {
            let id = self.next_polyp_id;
            self.next_polyp_id += 1;
            id
        }

        pub fn biodiversity_index(&self) -> f64 {
            if self.colonies.is_empty() { return 0.0; }
            let total: f64 = self.colonies.values().map(|c| c.population() as f64).sum();
            if total == 0.0 { return 0.0; }
            // Shannon diversity
            let mut h = 0.0;
            for colony in self.colonies.values() {
                let p = colony.population() as f64 / total;
                if p > 0.0 {
                    h -= p * p.ln();
                }
            }
            h
        }

        pub fn overall_health(&self) -> f64 {
            if self.colonies.is_empty() { return 0.0; }
            let healths: Vec<f64> = self.colonies.values().map(|c| c.average_health()).collect();
            healths.iter().sum::<f64>() / healths.len() as f64
        }

        pub fn species_list(&self) -> Vec<String> {
            self.colonies.values().map(|c| c.species.clone()).collect()
        }

        pub fn is_healthy(&self) -> bool {
            self.overall_health() > 0.5 && self.environment.is_healthy()
        }
    }
}

// ============================================================================
// symbiont module
// ============================================================================

pub mod symbiont {
    

    /// Type of symbiotic relationship.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum SymbiosisType {
        Mutualism,      // Both benefit
        Commensalism,   // One benefits, other neutral
        Parasitism,     // One benefits, other harmed
    }

    /// A symbiotic relationship between two agents.
    #[derive(Debug, Clone)]
    pub struct SymbiontPair {
        pub id_a: String,
        pub id_b: String,
        pub symbiosis_type: SymbiosisType,
        pub strength: f64,
        pub duration: u64,
    }

    impl SymbiontPair {
        pub fn new(id_a: &str, id_b: &str, stype: SymbiosisType, strength: f64) -> Self {
            Self {
                id_a: id_a.to_string(),
                id_b: id_b.to_string(),
                symbiosis_type: stype,
                strength: strength.clamp(0.0, 1.0),
                duration: 0,
            }
        }

        pub fn tick(&mut self) {
            self.duration += 1;
        }

        /// Apply symbiotic effects to health values.
        pub fn apply_effects(&self, health_a: &mut f64, health_b: &mut f64) {
            match self.symbiosis_type {
                SymbiosisType::Mutualism => {
                    *health_a = (*health_a + 0.01 * self.strength).min(1.0);
                    *health_b = (*health_b + 0.01 * self.strength).min(1.0);
                }
                SymbiosisType::Commensalism => {
                    *health_a = (*health_a + 0.01 * self.strength).min(1.0);
                }
                SymbiosisType::Parasitism => {
                    *health_a = (*health_a + 0.02 * self.strength).min(1.0);
                    *health_b = (*health_b - 0.02 * self.strength).max(0.0);
                }
            }
        }

        pub fn is_mutualistic(&self) -> bool {
            self.symbiosis_type == SymbiosisType::Mutualism
        }

        pub fn is_parasitic(&self) -> bool {
            self.symbiosis_type == SymbiosisType::Parasitism
        }

        pub fn is_beneficial_for(&self, id: &str) -> bool {
            match self.symbiosis_type {
                SymbiosisType::Mutualism => true,
                SymbiosisType::Commensalism => id == self.id_a,
                SymbiosisType::Parasitism => id == self.id_a,
            }
        }
    }

    /// Registry of all symbiotic relationships.
    #[derive(Debug, Clone)]
    pub struct SymbiontRegistry {
        pairs: Vec<SymbiontPair>,
    }

    impl SymbiontRegistry {
        pub fn new() -> Self {
            Self { pairs: Vec::new() }
        }

        pub fn add(&mut self, pair: SymbiontPair) {
            self.pairs.push(pair);
        }

        pub fn remove(&mut self, id_a: &str, id_b: &str) -> Option<SymbiontPair> {
            if let Some(pos) = self.pairs.iter().position(|p|
                (p.id_a == id_a && p.id_b == id_b) || (p.id_a == id_b && p.id_b == id_a)
            ) {
                Some(self.pairs.remove(pos))
            } else { None }
        }

        pub fn pairs_for(&self, id: &str) -> Vec<&SymbiontPair> {
            self.pairs.iter().filter(|p| p.id_a == id || p.id_b == id).collect()
        }

        pub fn tick_all(&mut self) {
            for pair in &mut self.pairs {
                pair.tick();
            }
        }

        pub fn pair_count(&self) -> usize {
            self.pairs.len()
        }

        pub fn mutualism_count(&self) -> usize {
            self.pairs.iter().filter(|p| p.is_mutualistic()).count()
        }

        pub fn parasitism_count(&self) -> usize {
            self.pairs.iter().filter(|p| p.is_parasitic()).count()
        }

        pub fn average_strength(&self) -> f64 {
            if self.pairs.is_empty() { return 0.0; }
            self.pairs.iter().map(|p| p.strength).sum::<f64>() / self.pairs.len() as f64
        }

        pub fn all_pairs(&self) -> &[SymbiontPair] {
            &self.pairs
        }

        pub fn find_pair(&self, id_a: &str, id_b: &str) -> Option<&SymbiontPair> {
            self.pairs.iter().find(|p|
                (p.id_a == id_a && p.id_b == id_b) || (p.id_a == id_b && p.id_b == id_a)
            )
        }
    }

    impl Default for SymbiontRegistry {
        fn default() -> Self { Self::new() }
    }
}

// ============================================================================
// bleaching module
// ============================================================================

pub mod bleaching {
    use super::colony::Colony;
    use super::reef::Environment;

    /// Stress response event for a colony.
    #[derive(Debug, Clone)]
    pub struct BleachingEvent {
        pub colony_id: String,
        pub severity: f64,     // 0.0 - 1.0
        pub trigger_temp: f64,
        pub duration: u64,
        pub recovering: bool,
    }

    impl BleachingEvent {
        pub fn new(colony_id: &str, severity: f64, trigger_temp: f64) -> Self {
            Self {
                colony_id: colony_id.to_string(),
                severity: severity.clamp(0.0, 1.0),
                trigger_temp,
                duration: 0,
                recovering: false,
            }
        }

        pub fn tick(&mut self) {
            self.duration += 1;
        }

        pub fn is_severe(&self) -> bool {
            self.severity > 0.7
        }

        pub fn is_mild(&self) -> bool {
            self.severity < 0.3
        }

        pub fn start_recovery(&mut self) {
            self.recovering = true;
        }

        pub fn recovery_progress(&self) -> f64 {
            if !self.recovering { return 0.0; }
            (self.duration as f64 / 100.0).min(1.0)
        }
    }

    /// Assess bleaching risk based on environment.
    pub fn assess_risk(env: &Environment) -> f64 {
        let temp_risk = if env.temperature > 29.0 {
            (env.temperature - 29.0) / 5.0
        } else if env.temperature < 18.0 {
            (18.0 - env.temperature) / 10.0
        } else {
            0.0
        };
        let light_risk = if env.light > 0.95 { 0.3 } else { 0.0 };
        let nutrient_risk = if env.nutrients < 0.2 { 0.2 } else { 0.0 };
        (temp_risk + light_risk + nutrient_risk).clamp(0.0, 1.0)
    }

    /// Apply bleaching damage to a colony based on severity.
    pub fn apply_bleaching(colony: &mut Colony, severity: f64) {
        for polyp in colony.polyps.values_mut() {
            polyp.damage(severity * 0.05);
        }
    }

    /// Attempt recovery based on improved conditions.
    pub fn attempt_recovery(colony: &mut Colony, env: &Environment, recovery_rate: f64) -> f64 {
        if env.is_healthy() {
            let health_before = colony.average_health();
            for polyp in colony.polyps.values_mut() {
                polyp.heal(recovery_rate * 0.02);
                polyp.feed(0.01);
            }
            colony.average_health() - health_before
        } else {
            0.0
        }
    }

    /// Generate a bleaching event from environment conditions.
    pub fn detect_bleaching(colony_id: &str, env: &Environment) -> Option<BleachingEvent> {
        let risk = assess_risk(env);
        if risk > 0.3 {
            Some(BleachingEvent::new(colony_id, risk, env.temperature))
        } else {
            None
        }
    }

    /// Check if a colony is in a bleaching state.
    pub fn is_bleached(colony: &Colony) -> bool {
        colony.average_health() < 0.3
    }

    /// Resilience score based on colony properties.
    pub fn resilience_score(colony: &Colony) -> f64 {
        let health = colony.average_health();
        let energy = colony.average_energy();
        let cohesion = colony.cohesion;
        let diversity = if colony.population() > 5 { 0.2 } else { 0.0 };
        health * 0.3 + energy * 0.3 + cohesion * 0.2 + diversity
    }
}

// Re-exports
pub use polyp::Polyp;
pub use colony::Colony;
pub use reef::{Reef, Environment};
pub use symbiont::{SymbiontPair, SymbiontRegistry, SymbiosisType};
pub use bleaching::{BleachingEvent, assess_risk, apply_bleaching, attempt_recovery, detect_bleaching, is_bleached, resilience_score};

#[cfg(test)]
mod tests {
    use super::*;

    // ---- polyp tests (14) ----

    #[test]
    fn test_polyp_new() {
        let p = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        assert_eq!(p.id, 1);
        assert_eq!(p.species, "coral");
        assert!((p.health - 1.0).abs() < 0.01);
        assert_eq!(p.age, 0);
    }

    #[test]
    fn test_polyp_with_health() {
        let p = polyp::Polyp::new(1, "coral", (0.0, 0.0)).with_health(0.5);
        assert!((p.health - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_polyp_with_energy() {
        let p = polyp::Polyp::new(1, "coral", (0.0, 0.0)).with_energy(0.3);
        assert!((p.energy - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_polyp_health_clamped() {
        let p = polyp::Polyp::new(1, "coral", (0.0, 0.0)).with_health(5.0);
        assert!(p.health <= 1.0);
    }

    #[test]
    fn test_polyp_traits() {
        let mut p = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        p.set_trait("growth_rate", 0.8);
        assert!((p.get_trait("growth_rate") - 0.8).abs() < 0.01);
        assert!((p.get_trait("nonexistent")).abs() < 0.01);
    }

    #[test]
    fn test_polyp_tick() {
        let mut p = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        p.tick();
        assert_eq!(p.age, 1);
        assert!(p.energy < 0.8);
    }

    #[test]
    fn test_polyp_feed() {
        let mut p = polyp::Polyp::new(1, "coral", (0.0, 0.0)).with_energy(0.5);
        p.feed(0.3);
        assert!((p.energy - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_polyp_damage_heal() {
        let mut p = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        p.damage(0.5);
        assert!((p.health - 0.5).abs() < 0.01);
        p.heal(0.3);
        assert!((p.health - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_polyp_is_alive() {
        let mut p = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        assert!(p.is_alive());
        p.damage(1.0);
        assert!(!p.is_alive());
    }

    #[test]
    fn test_polyp_distance() {
        let a = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        let b = polyp::Polyp::new(2, "coral", (3.0, 4.0));
        assert!((a.distance_to(&b) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_polyp_move_towards() {
        let mut p = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        p.move_towards((10.0, 0.0), 2.0);
        assert!((p.position.0 - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_polyp_fitness() {
        let p = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        assert!(p.fitness() > 0.5);
    }

    #[test]
    fn test_polyp_reproduce() {
        let p = polyp::Polyp::new(1, "coral", (5.0, 5.0)).with_energy(0.8);
        let child = p.reproduce(2);
        assert_eq!(child.id, 2);
        assert_eq!(child.species, "coral");
        assert!(child.energy < p.energy);
    }

    #[test]
    fn test_polyp_can_reproduce() {
        let p = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        assert!(!p.can_reproduce()); // age too young
        let mut p2 = polyp::Polyp::new(2, "coral", (0.0, 0.0));
        p2.age = 100;
        assert!(p2.can_reproduce());
    }

    // ---- colony tests (14) ----

    #[test]
    fn test_colony_new() {
        let c = colony::Colony::new("col1", "coral");
        assert_eq!(c.id, "col1");
        assert_eq!(c.species, "coral");
        assert!(c.is_empty());
    }

    #[test]
    fn test_colony_add_remove() {
        let mut c = colony::Colony::new("col1", "coral");
        let p = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        c.add(p);
        assert_eq!(c.population(), 1);
        c.remove(1);
        assert!(c.is_empty());
    }

    #[test]
    fn test_colony_get() {
        let mut c = colony::Colony::new("col1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)));
        assert!(c.get(1).is_some());
        assert!(c.get(99).is_none());
    }

    #[test]
    fn test_colony_average_health() {
        let mut c = colony::Colony::new("col1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)).with_health(1.0));
        c.add(polyp::Polyp::new(2, "coral", (0.0, 0.0)).with_health(0.5));
        assert!((c.average_health() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_colony_average_energy() {
        let mut c = colony::Colony::new("col1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)).with_energy(0.8));
        c.add(polyp::Polyp::new(2, "coral", (0.0, 0.0)).with_energy(0.4));
        assert!((c.average_energy() - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_colony_alive_count() {
        let mut c = colony::Colony::new("col1", "coral");
        let mut dead = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        dead.damage(1.0);
        c.add(dead);
        c.add(polyp::Polyp::new(2, "coral", (0.0, 0.0)));
        assert_eq!(c.alive_count(), 1);
    }

    #[test]
    fn test_colony_remove_dead() {
        let mut c = colony::Colony::new("col1", "coral");
        let mut dead = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        dead.damage(1.0);
        c.add(dead);
        c.add(polyp::Polyp::new(2, "coral", (0.0, 0.0)));
        let removed = c.remove_dead();
        assert_eq!(removed, 1);
        assert_eq!(c.population(), 1);
    }

    #[test]
    fn test_colony_tick_all() {
        let mut c = colony::Colony::new("col1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)));
        c.tick_all();
        assert_eq!(c.get(1).unwrap().age, 1);
    }

    #[test]
    fn test_colony_feed_all() {
        let mut c = colony::Colony::new("col1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)).with_energy(0.3));
        c.feed_all(0.2);
        assert!(c.get(1).unwrap().energy > 0.4);
    }

    #[test]
    fn test_colony_center_of_mass() {
        let mut c = colony::Colony::new("col1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)));
        c.add(polyp::Polyp::new(2, "coral", (2.0, 2.0)));
        let (cx, cy) = c.center_of_mass();
        assert!((cx - 1.0).abs() < 0.01);
        assert!((cy - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_colony_total_fitness() {
        let mut c = colony::Colony::new("col1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)));
        assert!(c.total_fitness() > 0.0);
    }

    #[test]
    fn test_colony_reproduction_candidates() {
        let mut c = colony::Colony::new("col1", "coral");
        let mut old_enough = polyp::Polyp::new(1, "coral", (0.0, 0.0));
        old_enough.age = 100;
        c.add(old_enough);
        c.add(polyp::Polyp::new(2, "coral", (0.0, 0.0)));
        assert_eq!(c.reproduction_candidates().len(), 1);
    }

    #[test]
    fn test_colony_cohesion() {
        let mut c = colony::Colony::new("col1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)));
        c.add(polyp::Polyp::new(2, "coral", (0.1, 0.1)));
        c.update_cohesion();
        assert!(c.cohesion > 0.5);
    }

    #[test]
    fn test_colony_polyp_ids() {
        let mut c = colony::Colony::new("col1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)));
        c.add(polyp::Polyp::new(3, "coral", (0.0, 0.0)));
        let ids = c.polyp_ids();
        assert_eq!(ids.len(), 2);
    }

    // ---- reef tests (10) ----

    #[test]
    fn test_reef_new() {
        let r = reef::Reef::new(reef::Environment::tropical());
        assert_eq!(r.colony_count(), 0);
        assert_eq!(r.tick_count, 0);
    }

    #[test]
    fn test_reef_add_colony() {
        let mut r = reef::Reef::new(reef::Environment::tropical());
        r.add_colony(colony::Colony::new("c1", "coral"));
        assert_eq!(r.colony_count(), 1);
    }

    #[test]
    fn test_reef_remove_colony() {
        let mut r = reef::Reef::new(reef::Environment::tropical());
        r.add_colony(colony::Colony::new("c1", "coral"));
        assert!(r.remove_colony("c1").is_some());
        assert_eq!(r.colony_count(), 0);
    }

    #[test]
    fn test_reef_total_population() {
        let mut r = reef::Reef::new(reef::Environment::tropical());
        let mut c1 = colony::Colony::new("c1", "coral");
        c1.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)));
        c1.add(polyp::Polyp::new(2, "coral", (0.0, 0.0)));
        r.add_colony(c1);
        assert_eq!(r.total_population(), 2);
    }

    #[test]
    fn test_reef_tick() {
        let mut r = reef::Reef::new(reef::Environment::tropical());
        let mut c = colony::Colony::new("c1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)));
        r.add_colony(c);
        r.tick();
        assert_eq!(r.tick_count, 1);
    }

    #[test]
    fn test_environment_stress() {
        let env = reef::Environment::optimal();
        assert!(env.stress_level() < 0.1);
        let bad_env = reef::Environment { temperature: 35.0, light: 0.3, nutrients: 0.1, ph: 6.0 };
        assert!(bad_env.stress_level() > 0.3);
    }

    #[test]
    fn test_biodiversity_index() {
        let mut r = reef::Reef::new(reef::Environment::tropical());
        let mut c1 = colony::Colony::new("c1", "coral");
        c1.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)));
        let mut c2 = colony::Colony::new("c2", "algae");
        c2.add(polyp::Polyp::new(2, "algae", (0.0, 0.0)));
        r.add_colony(c1);
        r.add_colony(c2);
        let bd = r.biodiversity_index();
        assert!(bd > 0.0);
    }

    #[test]
    fn test_overall_health() {
        let mut r = reef::Reef::new(reef::Environment::tropical());
        let mut c = colony::Colony::new("c1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)));
        r.add_colony(c);
        assert!(r.overall_health() > 0.5);
    }

    #[test]
    fn test_is_healthy() {
        let r = reef::Reef::new(reef::Environment::optimal());
        assert!(!r.is_healthy()); // no colonies
    }

    #[test]
    fn test_allocate_polyp_id() {
        let mut r = reef::Reef::new(reef::Environment::tropical());
        let id1 = r.allocate_polyp_id();
        let id2 = r.allocate_polyp_id();
        assert!(id2 > id1);
    }

    // ---- symbiont tests (14) ----

    #[test]
    fn test_symbiont_pair_new() {
        let sp = symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Mutualism, 0.8);
        assert_eq!(sp.id_a, "a");
        assert!(sp.is_mutualistic());
        assert!(!sp.is_parasitic());
    }

    #[test]
    fn test_symbiont_strength_clamped() {
        let sp = symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Mutualism, 5.0);
        assert!(sp.strength <= 1.0);
    }

    #[test]
    fn test_mutualism_effects() {
        let sp = symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Mutualism, 1.0);
        let mut ha = 0.5;
        let mut hb = 0.5;
        sp.apply_effects(&mut ha, &mut hb);
        assert!(ha > 0.5);
        assert!(hb > 0.5);
    }

    #[test]
    fn test_commensalism_effects() {
        let sp = symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Commensalism, 1.0);
        let mut ha = 0.5;
        let mut hb = 0.5;
        sp.apply_effects(&mut ha, &mut hb);
        assert!(ha > 0.5);
        assert!((hb - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_parasitism_effects() {
        let sp = symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Parasitism, 1.0);
        let mut ha = 0.5;
        let mut hb = 0.8;
        sp.apply_effects(&mut ha, &mut hb);
        assert!(ha > 0.5);
        assert!(hb < 0.8);
    }

    #[test]
    fn test_is_beneficial_for() {
        let sp = symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Parasitism, 0.5);
        assert!(sp.is_beneficial_for("a"));
        assert!(!sp.is_beneficial_for("b"));
    }

    #[test]
    fn test_symbiont_tick() {
        let mut sp = symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Mutualism, 0.5);
        sp.tick();
        assert_eq!(sp.duration, 1);
    }

    #[test]
    fn test_registry_add() {
        let mut reg = symbiont::SymbiontRegistry::new();
        reg.add(symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Mutualism, 0.5));
        assert_eq!(reg.pair_count(), 1);
    }

    #[test]
    fn test_registry_remove() {
        let mut reg = symbiont::SymbiontRegistry::new();
        reg.add(symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Mutualism, 0.5));
        assert!(reg.remove("a", "b").is_some());
        assert_eq!(reg.pair_count(), 0);
    }

    #[test]
    fn test_registry_pairs_for() {
        let mut reg = symbiont::SymbiontRegistry::new();
        reg.add(symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Mutualism, 0.5));
        reg.add(symbiont::SymbiontPair::new("a", "c", symbiont::SymbiosisType::Commensalism, 0.3));
        assert_eq!(reg.pairs_for("a").len(), 2);
        assert_eq!(reg.pairs_for("b").len(), 1);
    }

    #[test]
    fn test_registry_counts() {
        let mut reg = symbiont::SymbiontRegistry::new();
        reg.add(symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Mutualism, 0.5));
        reg.add(symbiont::SymbiontPair::new("c", "d", symbiont::SymbiosisType::Parasitism, 0.5));
        reg.add(symbiont::SymbiontPair::new("e", "f", symbiont::SymbiosisType::Mutualism, 0.5));
        assert_eq!(reg.mutualism_count(), 2);
        assert_eq!(reg.parasitism_count(), 1);
    }

    #[test]
    fn test_registry_average_strength() {
        let mut reg = symbiont::SymbiontRegistry::new();
        reg.add(symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Mutualism, 0.4));
        reg.add(symbiont::SymbiontPair::new("c", "d", symbiont::SymbiosisType::Mutualism, 0.6));
        assert!((reg.average_strength() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_registry_tick_all() {
        let mut reg = symbiont::SymbiontRegistry::new();
        reg.add(symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Mutualism, 0.5));
        reg.tick_all();
        assert_eq!(reg.all_pairs()[0].duration, 1);
    }

    #[test]
    fn test_registry_find_pair() {
        let mut reg = symbiont::SymbiontRegistry::new();
        reg.add(symbiont::SymbiontPair::new("a", "b", symbiont::SymbiosisType::Mutualism, 0.5));
        assert!(reg.find_pair("a", "b").is_some());
        assert!(reg.find_pair("b", "a").is_some());
        assert!(reg.find_pair("a", "c").is_none());
    }

    // ---- bleaching tests (10) ----

    #[test]
    fn test_bleaching_event_new() {
        let be = bleaching::BleachingEvent::new("col1", 0.8, 30.0);
        assert_eq!(be.colony_id, "col1");
        assert!(be.is_severe());
        assert!(!be.is_mild());
    }

    #[test]
    fn test_bleaching_event_mild() {
        let be = bleaching::BleachingEvent::new("col1", 0.2, 28.0);
        assert!(be.is_mild());
        assert!(!be.is_severe());
    }

    #[test]
    fn test_bleaching_event_severity_clamped() {
        let be = bleaching::BleachingEvent::new("col1", 5.0, 30.0);
        assert!(be.severity <= 1.0);
    }

    #[test]
    fn test_bleaching_event_tick() {
        let mut be = bleaching::BleachingEvent::new("col1", 0.5, 30.0);
        be.tick();
        assert_eq!(be.duration, 1);
    }

    #[test]
    fn test_bleaching_recovery() {
        let mut be = bleaching::BleachingEvent::new("col1", 0.5, 30.0);
        be.start_recovery();
        assert!(be.recovering);
        assert_eq!(be.recovery_progress(), 0.0); // just started
    }

    #[test]
    fn test_assess_risk_low() {
        let env = reef::Environment::optimal();
        let risk = bleaching::assess_risk(&env);
        assert!(risk < 0.2);
    }

    #[test]
    fn test_assess_risk_high_temp() {
        let env = reef::Environment { temperature: 32.0, light: 0.8, nutrients: 0.6, ph: 8.1 };
        let risk = bleaching::assess_risk(&env);
        assert!(risk > 0.3);
    }

    #[test]
    fn test_apply_bleaching() {
        let mut c = colony::Colony::new("col1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)));
        bleaching::apply_bleaching(&mut c, 0.5);
        assert!(c.average_health() < 1.0);
    }

    #[test]
    fn test_detect_bleaching_none() {
        let env = reef::Environment::optimal();
        let result = bleaching::detect_bleaching("col1", &env);
        assert!(result.is_none());
    }

    #[test]
    fn test_detect_bleaching_some() {
        let env = reef::Environment { temperature: 33.0, light: 0.8, nutrients: 0.6, ph: 8.1 };
        let result = bleaching::detect_bleaching("col1", &env);
        assert!(result.is_some());
    }

    #[test]
    fn test_is_bleached() {
        let mut c = colony::Colony::new("col1", "coral");
        c.add(polyp::Polyp::new(1, "coral", (0.0, 0.0)).with_health(0.2));
        assert!(bleaching::is_bleached(&c));
    }

    #[test]
    fn test_resilience_score() {
        let mut c = colony::Colony::new("col1", "coral");
        for i in 0..10 {
            c.add(polyp::Polyp::new(i, "coral", (i as f64 * 0.1, 0.0)));
        }
        let score = bleaching::resilience_score(&c);
        assert!(score > 0.0 && score <= 1.0);
    }
}
