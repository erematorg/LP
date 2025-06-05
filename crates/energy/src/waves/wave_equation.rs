use bevy::prelude::*;

/// 2D Wave equation solver (∂²u/∂t² = c²(∂²u/∂x² + ∂²u/∂y²))
#[derive(Debug, Clone, Reflect)]
pub struct WaveEquation2D {
    /// Grid dimensions
    pub nx: usize,
    pub ny: usize,
    /// Wave speed
    pub c: f32,
    /// Grid spacing
    pub dx: f32,
    pub dy: f32,
    /// Time step
    pub dt: f32,
    /// Current solution u(t)
    pub u_current: Vec<f32>,
    /// Previous solution u(t-dt)
    pub u_previous: Vec<f32>,
    /// Pre-computed coefficient for x-laplacian
    laplacian_x_coeff: f32,
    /// Pre-computed coefficient for y-laplacian
    laplacian_y_coeff: f32,
}

impl WaveEquation2D {
    pub fn new(nx: usize, ny: usize, c: f32, dx: f32, dy: f32, dt: f32) -> Self {
        // Initialize with zero arrays
        let u_current = vec![0.0; nx * ny];
        let u_previous = vec![0.0; nx * ny];

        // Pre-compute coefficient for wave equation
        let laplacian_x_coeff = (c * dt / dx).powi(2);
        let laplacian_y_coeff = (c * dt / dy).powi(2);

        Self {
            nx,
            ny,
            c,
            dx,
            dy,
            dt,
            u_current,
            u_previous,
            laplacian_x_coeff,
            laplacian_y_coeff,
        }
    }

    /// Check stability (Courant condition)
    pub fn is_stable(&self) -> bool {
        let courant = self.c * self.dt * ((1.0 / self.dx).powi(2) + (1.0 / self.dy).powi(2)).sqrt();
        courant <= 1.0
    }

    /// Set initial conditions
    pub fn set_initial_displacement(&mut self, u0: Vec<f32>) {
        if u0.len() == self.nx * self.ny {
            self.u_current = u0.clone();
            self.u_previous = u0;
        }
    }

    /// Get value at specific grid point
    /// Uses row-major layout (y*nx+x) for better cache locality during inner loop iteration
    #[inline]
    fn get(&self, grid: &[f32], x: usize, y: usize) -> f32 {
        grid[y * self.nx + x]
    }

    /// Set value at specific grid point
    /// Uses row-major layout (y*nx+x) for better cache locality during inner loop iteration
    #[inline]
    fn set(&mut self, grid: &mut [f32], x: usize, y: usize, value: f32) {
        grid[y * self.nx + x] = value;
    }

    /// Solve one time step using finite difference method
    pub fn step(&mut self) {
        // Create next state
        let mut u_next = vec![0.0; self.nx * self.ny];

        // Use pre-computed coefficients
        let cx = self.laplacian_x_coeff;
        let cy = self.laplacian_y_coeff;

        // Apply wave equation to interior points
        // Process in chunks for better cache usage
        const CHUNK_SIZE: usize = 32;

        // Optimize memory access patterns by processing in blocks
        for j_chunk in 1..((self.ny - 1).div_ceil(CHUNK_SIZE)) {
            let j_start = (j_chunk - 1) * CHUNK_SIZE + 1;
            let j_end = (j_start + CHUNK_SIZE - 1).min(self.ny - 2);

            for i_chunk in 1..((self.nx - 1).div_ceil(CHUNK_SIZE)) {
                let i_start = (i_chunk - 1) * CHUNK_SIZE + 1;
                let i_end = (i_start + CHUNK_SIZE - 1).min(self.nx - 2);

                for j in j_start..=j_end {
                    for i in i_start..=i_end {
                        // Finite difference formula for 2D wave equation
                        let laplacian_x = self.get(&self.u_current, i + 1, j)
                            - 2.0 * self.get(&self.u_current, i, j)
                            + self.get(&self.u_current, i - 1, j);

                        let laplacian_y = self.get(&self.u_current, i, j + 1)
                            - 2.0 * self.get(&self.u_current, i, j)
                            + self.get(&self.u_current, i, j - 1);

                        let next_value = 2.0 * self.get(&self.u_current, i, j)
                            - self.get(&self.u_previous, i, j)
                            + cx * laplacian_x
                            + cy * laplacian_y;

                        self.set(&mut u_next, i, j, next_value);
                    }
                }
            }
        }

        // Apply fixed boundary conditions (u=0 at boundary)
        for i in 0..self.nx {
            self.set(&mut u_next, i, 0, 0.0);
            self.set(&mut u_next, i, self.ny - 1, 0.0);
        }

        for j in 0..self.ny {
            self.set(&mut u_next, 0, j, 0.0);
            self.set(&mut u_next, self.nx - 1, j, 0.0);
        }

        // Update states
        self.u_previous = self.u_current.clone();
        self.u_current = u_next;
    }
}

/// Component wrapper for the wave equation
#[derive(Component, Reflect)]
pub struct WaveEquationComponent {
    pub solver: WaveEquation2D,
}

/// System to update wave equation simulation
pub fn update_wave_equation(mut query: Query<&mut WaveEquationComponent>) {
    for mut component in query.iter_mut() {
        component.solver.step();
    }
}
