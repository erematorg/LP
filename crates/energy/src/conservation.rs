use bevy::prelude::*;

/// Enum representing different types of energy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component, Reflect)]
pub enum EnergyType {
    Generic,
    Thermal,
    Kinetic,
    Potential,
    Chemical,
    Electromagnetic,
    Solar,
}

/// Component tracking energy in a system
#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct EnergyQuantity {
    /// Energy amount in joules
    pub value: f32,
    /// Type of energy
    pub energy_type: EnergyType,
    /// Maximum energy capacity (optional)
    pub max_capacity: Option<f32>,
}

impl EnergyQuantity {
    /// Create a new energy quantity
    pub fn new(value: f32, energy_type: EnergyType, max_capacity: Option<f32>) -> Self {
        let clamped_value = max_capacity.map(|max| value.min(max)).unwrap_or(value);

        Self {
            value: clamped_value.max(0.0),
            energy_type,
            max_capacity,
        }
    }

    /// Add energy, respecting max capacity
    pub fn add(&mut self, amount: f32) {
        if let Some(max) = self.max_capacity {
            self.value = (self.value + amount).min(max);
        } else {
            self.value += amount;
        }
    }

    /// Subtract energy, preventing negative values
    pub fn subtract(&mut self, amount: f32) {
        self.value = (self.value - amount).max(0.0);
    }
}

/// Energy transaction types for conservation accounting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum TransactionType {
    Input,  // Energy entering the system
    Output, // Energy leaving the system
}

/// Event for energy transfers between entities
#[derive(Event, Debug)]
pub struct EnergyTransferEvent {
    /// Source entity losing energy
    pub source: Entity,
    /// Target entity receiving energy
    pub target: Entity,
    /// Amount of energy to transfer
    pub amount: f32,
    /// Type of energy being transferred
    pub energy_type: EnergyType,
}

/// Component for precise energy accounting
#[derive(Component, Debug, Reflect)]
pub struct EnergyAccountingLedger {
    /// History of all transactions, newest first
    pub transactions: Vec<EnergyTransaction>,
    /// Maximum number of transactions to store
    pub max_history: usize,
    /// Sum of all inputs
    pub total_input: f32,
    /// Sum of all outputs
    pub total_output: f32,
}

/// Record of a single energy transaction
#[derive(Debug, Clone, Reflect)]
pub struct EnergyTransaction {
    /// Type of transaction
    pub transaction_type: TransactionType,
    /// Amount of energy involved (joules)
    pub amount: f32,
    /// Source of energy (None for inputs from outside system)
    pub source: Option<Entity>,
    /// Destination of energy (None for outputs to outside system)
    pub destination: Option<Entity>,
    /// Timestamp when the transaction occurred
    pub timestamp: f32,
}

impl Default for EnergyAccountingLedger {
    fn default() -> Self {
        Self {
            transactions: Vec::new(),
            max_history: 100,
            total_input: 0.0,
            total_output: 0.0,
        }
    }
}

impl EnergyAccountingLedger {
    /// Record a new energy transaction
    pub fn record_transaction(&mut self, transaction: EnergyTransaction) {
        match transaction.transaction_type {
            TransactionType::Input => self.total_input += transaction.amount,
            TransactionType::Output => self.total_output += transaction.amount,
        }

        self.transactions.insert(0, transaction);
        if self.transactions.len() > self.max_history {
            self.transactions.pop();
        }
    }

    /// Get the net energy change
    pub fn net_energy_change(&self) -> f32 {
        self.total_input - self.total_output
    }
}

/// Resource for tracking energy conservation across systems
#[derive(Resource, Debug)]
pub struct EnergyConservationTracker {
    /// Total energy of the system at initialization
    pub initial_total_energy: f32,
    /// Maximum allowed conservation error
    pub tolerance: f32,
}

impl Default for EnergyConservationTracker {
    fn default() -> Self {
        Self {
            initial_total_energy: 0.0,
            tolerance: 1e-6, // Small tolerance for floating-point comparisons
        }
    }
}

/// Utility function to verify energy conservation
pub fn verify_conservation(initial_energy: f32, final_energy: f32, tolerance: f32) -> bool {
    // First law: Energy cannot be created or destroyed
    (final_energy - initial_energy).abs() <= tolerance
}

/// Utility function to calculate conversion efficiency
pub fn conversion_efficiency(energy_input: f32, energy_output: f32) -> f32 {
    if energy_input > 0.0 {
        (energy_output / energy_input).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Plugin to manage energy conservation systems
pub struct EnergyConservationPlugin;

impl Plugin for EnergyConservationPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register types for reflection
            .register_type::<EnergyType>()
            .register_type::<EnergyQuantity>()
            .register_type::<TransactionType>()
            .register_type::<EnergyTransaction>()
            .register_type::<EnergyAccountingLedger>()
            // Add resources
            .init_resource::<EnergyConservationTracker>()
            // Add event channel
            .add_event::<EnergyTransferEvent>();
    }
}
