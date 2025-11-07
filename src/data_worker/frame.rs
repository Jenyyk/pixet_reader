use crate::data_worker::particle::Particle;
use std::collections::{HashMap, HashSet};

#[derive(Default, Debug)]
pub struct Frame {
    pub data: Vec<Vec<i16>>,
    particles: Vec<Particle>,
}

impl Frame {
    pub fn new(data: Vec<Vec<i16>>) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }

    pub fn get_particles(&self) -> Vec<Particle> {
        self.particles.clone()
    }

    pub fn count_particles(&mut self, mut kernel_size: usize) {
        if kernel_size % 2 == 0 {
            kernel_size += 1;
        }

        let mut labeled_data: Vec<Vec<(i16, Vec<u8>)>> = self
            .data
            .clone()
            .into_iter()
            .map(|row| row.into_iter().map(|point| (point, Vec::new())).collect())
            .collect();

        if labeled_data.is_empty() || labeled_data[0].is_empty() {
            return;
        }

        let mut label_counter: u8 = 1;
        let rows = labeled_data.len();
        let cols = labeled_data[0].len();
        let kernel_radius = (kernel_size / 2) as isize;

        // Disjoint Set Union (DSU) for merging labels
        let mut parent: Vec<u8> = (0..=255).collect();
        fn find(p: &mut [u8], i: u8) -> u8 {
            if p[i as usize] == i {
                i
            } else {
                p[i as usize] = find(p, p[i as usize]);
                p[i as usize]
            }
        }
        fn union(p: &mut [u8], i: u8, j: u8) {
            let root_i = find(p, i);
            let root_j = find(p, j);
            if root_i != root_j {
                p[root_j as usize] = root_i;
            }
        }

        // First pass: scan and apply labels
        for i in 0..rows {
            for j in 0..cols {
                if labeled_data[i][j].0 == 0 {
                    continue;
                }

                let mut neighbor_labels = HashSet::new();

                // Check neighbors within the kernel that have already been processed.
                for dy in -kernel_radius..=kernel_radius {
                    for dx in -kernel_radius..=kernel_radius {
                        if dy == 0 && dx == 0 {
                            continue; // Skip the current pixel
                        }

                        let ni = i as isize + dy;
                        let nj = j as isize + dx;

                        // Check if the neighbor is within bounds and has been processed
                        if ni >= 0 && ni < rows as isize && nj >= 0 && nj < cols as isize {
                            if ni < i as isize || (ni == i as isize && nj < j as isize) {
                                let ni = ni as usize;
                                let nj = nj as usize;
                                if !labeled_data[ni][nj].1.is_empty() {
                                    neighbor_labels.extend(labeled_data[ni][nj].1.iter());
                                }
                            }
                        }
                    }
                }

                if neighbor_labels.is_empty() {
                    // This is a new component, so we assign a new label.
                    labeled_data[i][j].1.push(label_counter);
                    if label_counter < 255 {
                        label_counter += 1;
                    }
                } else {
                    // This pixel is connected to one or more existing components.
                    // We apply all unique neighbor labels to this pixel.
                    let labels: Vec<u8> = neighbor_labels.into_iter().copied().collect();
                    labeled_data[i][j].1.extend(&labels);

                    // We then merge these labels to note that they are all part of the same component.
                    if labels.len() > 1 {
                        let first = labels[0];
                        for &other in labels.iter().skip(1) {
                            union(&mut parent, first, other);
                        }
                    }
                }
            }
        }

        // Second pass: Group pixels by their root label.
        let mut particle_pixels: HashMap<u8, Vec<(usize, usize, i16)>> = HashMap::new();

        for i in 0..rows {
            for j in 0..cols {
                if let Some(&label) = labeled_data[i][j].1.first() {
                    let root_label = find(&mut parent, label);
                    let pixel_data = (i, j, labeled_data[i][j].0);
                    particle_pixels
                        .entry(root_label)
                        .or_default()
                        .push(pixel_data);
                }
            }
        }

        self.particles = particle_pixels
            .into_iter()
            .map(|(_id, pixels)| {
                Particle::new(pixels)
            })
            .collect();
    }
}
