#![feature(portable_simd)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::simd::u64x4;
use std::simd::num::SimdUint;

#[derive(Debug, PartialEq)]
pub enum QError {
    VoidLeak,
    OverCarved,
    ThermalJitter,
    SourceDensityLow,
    PatternInjection,
    InstructionOverlap,
}

pub struct QEngineV2 {
    torus_flux: AtomicU64,
    last_mask: u64,
    reclaimer_buffer: f64,
}

impl QEngineV2 {
    pub fn new(initial_entropy: u64) -> Self {
        Self {
            torus_flux: AtomicU64::new(initial_entropy),
            last_mask: 0,
            reclaimer_buffer: 0.0,
        }
    }

    pub fn update_entropy(&self, new_val: u64) {
        self.torus_flux.store(new_val, Ordering::Release);
    }

    pub fn execute_carve(&mut self, instruction_set: &[u64]) -> Result<u64, QError> {
        let entropy_snapshot = self.torus_flux.load(Ordering::Acquire);
        self.fast_mask_audit(instruction_set)?;
        self.audit_source(entropy_snapshot)?;

        let mut final_output: u64 = 0;
        let mut reclaimed_bits: u64 = 0;

        for &mask in instruction_set {
            self.limit_thermal_drift(mask)?;
            let carved_result = entropy_snapshot & mask;
            if self.validate_sculpt(carved_result, entropy_snapshot) {
                final_output |= carved_result;
            } else {
                reclaimed_bits |= carved_result;
            }
        }

        self.process_reclamation(reclaimed_bits);
        self.verify_mass_conservation(entropy_snapshot, final_output, reclaimed_bits, instruction_set)?;
        self.last_mask = final_output;
        Ok(final_output)
    }

    fn fast_mask_audit(&self, masks: &[u64]) -> Result<(), QError> {
        let mut chunks = masks.chunks_exact(4);
        for chunk in chunks.by_ref() {
            let m0 = chunk[0];
            let m1 = chunk[1];
            let m2 = chunk[2];
            let m3 = chunk[3];
            if (m0 & m1) != 0 || ((m0 | m1) & m2) != 0 || ((m0 | m1 | m2) & m3) != 0 {
                return Err(QError::InstructionOverlap);
            }
        }
        let remainder = chunks.remainder();
        if !remainder.is_empty() {
            let mut check_mask = 0;
            for &m in remainder {
                if (check_mask & m) != 0 {
                    return Err(QError::InstructionOverlap);
                }
                check_mask |= m;
            }
        }
        Ok(())
    }

    fn validate_sculpt(&self, carved: u64, source: u64) -> bool {
        let source_weight = source.count_ones();
        if source_weight == 0 { return false; }
        let ratio = carved.count_ones() as f32 / source_weight as f32;
        ratio > 0.001 && ratio < 0.999
    }

    fn limit_thermal_drift(&self, current_mask: u64) -> Result<(), QError> {
        let diff = (current_mask ^ self.last_mask).count_ones();
        if diff > 48 { return Err(QError::ThermalJitter); }
        Ok(())
    }

    fn process_reclamation(&mut self, bits: u64) {
        let bit_count = bits.count_ones() as f64;
        self.reclaimer_buffer += bit_count.ln_1p();
    }

    fn audit_source(&self, source: u64) -> Result<(), QError> {
        if source.count_ones() < 4 { return Err(QError::SourceDensityLow); }
        if source == 0xAAAAAAAAAAAAAAAA || source == 0x5555555555555555 {
            return Err(QError::PatternInjection);
        }
        Ok(())
    }

    fn verify_mass_conservation(&self, source: u64, out: u64, rec: u64, masks: &[u64]) -> Result<(), QError> {
        let combined_mask = masks.iter().fold(0, |acc, &m| acc | m);
        let active_source = source & combined_mask;
        if (out | rec) != active_source { return Err(QError::VoidLeak); }
        Ok(())
    }
}
