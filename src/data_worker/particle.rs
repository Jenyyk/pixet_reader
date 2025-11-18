#[derive(Debug, Clone)]
pub struct Particle {
    pub particle_type: ParticleType,
    pub positions: Vec<(usize, usize, i16)>,
}

impl Particle {
    pub fn new(positions: Vec<(usize, usize, i16)>) -> Self {
        Particle {
            particle_type: ParticleType::Unknown,
            positions,
        }
    }

    pub fn calculate_type(&mut self) {
        let left_most_coord = self.positions.iter().min_by_key(|pos| pos.0).unwrap().0;
        let right_most_coord = self.positions.iter().max_by_key(|pos| pos.0).unwrap().0;
        let top_most_coord = self.positions.iter().min_by_key(|pos| pos.1).unwrap().1;
        let bottom_most_coord = self.positions.iter().max_by_key(|pos| pos.1).unwrap().1;

        let size = usize::max(
            right_most_coord - left_most_coord,
            bottom_most_coord - top_most_coord,
        );

        if size >= 12 {
            self.particle_type = ParticleType::PossibleMuon(size);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParticleType {
    PossibleMuon(usize),
    Unknown,
}
