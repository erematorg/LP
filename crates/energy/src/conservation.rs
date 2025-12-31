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
#[reflect(Component)]
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
        debug_assert!(
            value >= 0.0,
            "Energy cannot be negative (violates conservation)"
        );
        debug_assert!(
            value < 1e20,
            "Energy exceeds realistic bounds (nuclear scale ~1e20 J)"
        );
        if let Some(max) = max_capacity {
            debug_assert!(max > 0.0, "Energy capacity must be positive");
        }

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
#[derive(Message, Debug)]
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
#[reflect(Component)]
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
    /// Transfer rate (Watts = Joules/second) - for flux tracking
    pub transfer_rate: f32,
    /// Duration of the transfer (seconds) - for sustained flows
    pub duration: f32,
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

    /// Calculate average transfer rate from recent transactions
    pub fn average_transfer_rate(&self, time_window: f32) -> f32 {
        if self.transactions.is_empty() {
            return 0.0;
        }

        let current_time = self
            .transactions
            .first()
            .map(|t| t.timestamp)
            .unwrap_or(0.0);
        let cutoff_time = current_time - time_window;

        let mut total_rate = 0.0;
        let mut count = 0;

        for transaction in self
            .transactions
            .iter()
            .take_while(|t| t.timestamp >= cutoff_time)
        {
            total_rate += transaction.transfer_rate;
            count += 1;
        }

        if count == 0 {
            0.0
        } else {
            total_rate / count as f32
        }
    }

    /// Get current energy flux (sum of all active transfer rates)
    pub fn current_flux(&self, current_time: f32, active_duration: f32) -> f32 {
        let cutoff_time = current_time - active_duration;

        self.transactions
            .iter()
            .filter(|t| t.timestamp >= cutoff_time && t.duration > 0.0)
            .map(|t| t.transfer_rate)
            .sum()
    }

    /// Find transactions involving a specific entity
    pub fn transactions_with_entity(&self, entity: Entity) -> Vec<&EnergyTransaction> {
        self.transactions
            .iter()
            .filter(|t| t.source == Some(entity) || t.destination == Some(entity))
            .collect()
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

/// Optional resource to monitor energy drift over time (diagnostic only, not enforced)
#[derive(Resource, Debug)]
pub struct EnergyDriftMonitor {
    pub initial_energy: f32,
    pub tolerance: f32,
}

impl EnergyDriftMonitor {
    pub fn new(initial_energy: f32, tolerance: f32) -> Self {
        Self {
            initial_energy,
            tolerance,
        }
    }

    /// Check if current energy has drifted beyond tolerance
    /// Returns Some(drift) if drift exceeds tolerance, None otherwise
    pub fn check_drift(&self, current_energy: f32) -> Option<f32> {
        let drift = (current_energy - self.initial_energy).abs();
        if drift > self.tolerance {
            Some(drift)
        } else {
            None
        }
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
            .add_message::<EnergyTransferEvent>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_arithmetic() {
        // Test that net_energy_change = total_input - total_output
        let mut ledger = EnergyAccountingLedger::default();

        ledger.record_transaction(EnergyTransaction {
            transaction_type: TransactionType::Input,
            amount: 100.0,
            source: None,
            destination: None,
            timestamp: 0.0,
            transfer_rate: 0.0,
            duration: 0.0,
        });

        ledger.record_transaction(EnergyTransaction {
            transaction_type: TransactionType::Output,
            amount: 30.0,
            source: None,
            destination: None,
            timestamp: 0.0,
            transfer_rate: 0.0,
            duration: 0.0,
        });

        ledger.record_transaction(EnergyTransaction {
            transaction_type: TransactionType::Input,
            amount: 50.0,
            source: None,
            destination: None,
            timestamp: 0.0,
            transfer_rate: 0.0,
            duration: 0.0,
        });

        assert_eq!(ledger.total_input, 150.0);
        assert_eq!(ledger.total_output, 30.0);
        assert_eq!(ledger.net_energy_change(), 120.0);
    }

    #[test]
    fn test_current_flux_sums_active_rates() {
        // Test that current_flux sums transfer rates correctly
        let mut ledger = EnergyAccountingLedger::default();

        let current_time = 10.0;

        // Add active transfer (within time window)
        ledger.record_transaction(EnergyTransaction {
            transaction_type: TransactionType::Input,
            amount: 50.0,
            source: None,
            destination: None,
            timestamp: 9.5,
            transfer_rate: 10.0, // W
            duration: 1.0,
        });

        // Add another active transfer
        ledger.record_transaction(EnergyTransaction {
            transaction_type: TransactionType::Input,
            amount: 30.0,
            source: None,
            destination: None,
            timestamp: 9.8,
            transfer_rate: 5.0, // W
            duration: 0.5,
        });

        // Add old transfer (outside time window)
        ledger.record_transaction(EnergyTransaction {
            transaction_type: TransactionType::Input,
            amount: 100.0,
            source: None,
            destination: None,
            timestamp: 5.0,
            transfer_rate: 20.0, // W
            duration: 2.0,
        });

        let flux = ledger.current_flux(current_time, 1.0);
        assert_eq!(flux, 15.0, "Expected sum of active rates: 10.0 + 5.0");
    }
}
