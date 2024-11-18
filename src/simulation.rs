use rand::Rng;

#[derive(Clone)]
pub struct SimulationState {
    pub particle_count: u32,
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
    pub accelerations: Vec<[f32; 3]>,
    pub masses: Vec<f32>,
    pub g: f32,
}

impl SimulationState {
    pub fn new(particle_count: u32) -> Self {
        let mut rng = rand::thread_rng();
        let positions : Vec<[f32; 3]> = (0..particle_count)
            .map(|_| {
                let r = f32::powf(rng.gen_range(0.0..0.9), 0.1);
                let theta = rng.gen_range(0.0..std::f32::consts::TAU);
                [r * theta.cos(), r * theta.sin(), 0.0]
            })
            .collect();

        let velocities = positions
            .iter()
            .map(|[x, y, _]| {
                let speed = rng.gen_range(0.004..0.008);
                let magnitude = f32::sqrt(x * x + y * y);
                [-y / magnitude * speed, x / magnitude * speed, 0.0]
            })
            .collect();

        Self {
            particle_count,
            positions,
            velocities,
            accelerations: vec![[0.0; 3]; particle_count as usize],
            masses: vec![1000.0; particle_count as usize],
            g: 6.67430e-11,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.update_leapfrog(dt);
    }

    fn update_acceleration(&mut self) {
        // F = (G*m1m2/(r*r)) * (unit vector)
        // F = ma
        // a = G*m2/(r*r) * unit vector
        let softening = 1e-4;

        for p1 in 0..self.particle_count as usize {
            let mut acceleration = [0.0, 0.0, 0.0];
            let p1_pos = self.positions[p1];

            for p2 in 0..self.particle_count as usize {
                if p1 == p2 {
                    continue;
                }

                let p2_pos = self.positions[p2];
                let p2_mass = self.masses[p2];

                let dx = p2_pos[0] - p1_pos[0];
                let dy = p2_pos[1] - p1_pos[1];
                let dz = p2_pos[2] - p1_pos[2];
                let dist_sqr = dx * dx + dy * dy + dz * dz + softening;

                let inv_dist = 1.0 / f32::sqrt(dist_sqr);
                let inv_dist3 = inv_dist * inv_dist * inv_dist;

                let force = self.g * p2_mass * inv_dist3;

                acceleration[0] += force * dx;
                acceleration[1] += force * dy;
                acceleration[2] += force * dz;
            }

            self.accelerations[p1][0] = acceleration[0];
            self.accelerations[p1][1] = acceleration[1];
            self.accelerations[p1][2] = acceleration[2];
        }
    }

    #[allow(dead_code)]
    fn update_euler(&mut self, dt: f32) {
        self.update_acceleration();

        for p1 in 0..self.particle_count as usize {
            for i in 0..3 {
                self.velocities[p1][i] += self.accelerations[p1][i] * dt;
                self.positions[p1][i] += self.velocities[p1][i] * dt;
            }
        }
    }

    #[allow(dead_code)]
    fn update_leapfrog(&mut self, dt: f32) {
        for p1 in 0..self.particle_count as usize {
            for i in 0..3 {
                self.velocities[p1][i] += self.accelerations[p1][i] * dt * 0.5;
                self.positions[p1][i] += self.velocities[p1][i] * dt;
            }
        }

        self.update_acceleration();

        for p1 in 0..self.particle_count as usize {
            for i in 0..3 {
                self.velocities[p1][i] += self.accelerations[p1][i] * dt * 0.5;
            }
        }
    }
}
