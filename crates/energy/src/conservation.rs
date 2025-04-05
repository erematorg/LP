use bevy::prelude::*;

/// Component tracking energy in a system
#[derive(Component, Debug, Clone, Copy)]
pub struct EnergyQuantity {
    /// Energy amount in joules
    pub value: f32,
}

/// Component representing energy transfer between entities
#[derive(Component, Debug)]
pub struct EnergyTransfer {
    /// Amount of energy transferred (joules)
    pub amount: f32,
    /// Source entity
    pub source: Entity,
    /// Target entity
    pub target: Entity,
}

/// Generic component for tracking energy transformations
/// Each specific energy module can define its own implementation
#[derive(Component, Debug)]
pub struct EnergyConversion {
    /// Efficiency of the conversion process (0.0-1.0)
    pub efficiency: f32,
    /// Energy lost during conversion (joules)
    pub losses: f32,
}

/// Energy transaction types for conservation accounting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Input,   // Energy entering the system
    Output,  // Energy leaving the system
}

/// Record of a single energy transaction
#[derive(Debug, Clone)]
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

/// Component for precise energy accounting
#[derive(Component, Debug)]
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
    
    /// Get the net energy change for this entity
    pub fn net_energy_change(&self) -> f32 {
        self.total_input - self.total_output
    }
}

/// System for tracking energy conservation across entire systems
#[derive(Resource)]
pub struct SystemConservationTracker {
    /// Total energy of the system at initialization
    pub initial_total_energy: f32,
    /// Maximum allowed conservation error
    pub tolerance: f32,
}

/// Verify energy conservation in a closed system
pub fn verify_conservation(
    initial_energy: f32,
    final_energy: f32,
    tolerance: f32,
) -> bool {
    // First law: Energy cannot be created or destroyed
    (final_energy - initial_energy).abs() <= tolerance
}

/// Calculate efficiency of an energy conversion process
pub fn conversion_efficiency(
    energy_input: f32,
    energy_output: f32,
) -> f32 {
    if energy_input > 0.0 {
        (energy_output / energy_input).clamp(0.0, 1.0)
    } else {
        0.0
    }
}