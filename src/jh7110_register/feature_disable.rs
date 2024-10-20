#![allow(dead_code)]

use crate::{clear_csr, read_as_csr, set_clear_csr, set_csr, write_as_csr};
// feature_disable register
const FEATURE_DISABLE_CSR_ADDRESS: usize = 0x7C1;
/// Bit-field mask that represents the settable fields in the [FeatureDisable] CSR.
pub const FIELD_MASK: usize = 0b11_0000_0010_0000_1111;
//BIT   DESCRIPTION
// 0        Disable data cache clock gating
// 1        Disable instruction cache clock gating
// 2        Disable pipeline clock gating
// 3        Disable speculative instruction cache refill
// 8:4      Reserved
// 9        Suppress corrupt signal on GrantData messages
// 15:10    Reserved
// 16       Disable short forward branch optimization
// 17       Disable instruction cache next-line prefetcher
// 63:18    Reserved

const DISABLE_DATA_CACHE_CLOCK_GATING: usize = 0;
const DISABLE_INSTRUCTION_CACHE_CLOCK_GATING: usize = 1;
const DISABLE_PIPELINE_CLOCK_GATING: usize = 2;
const DISABLE_SPECULATIVE_INSTRUCTION_CACHE_REFILL: usize = 3;
const SUPPRESS_CORRUPT_SIGNAL_ON_GRANTDATA_MESSAGE: usize = 9;
const DISABLE_SHORT_FORWARD_BRANCH_OPTIMIZATION: usize = 16;
const DISABLE_INSTRUCTION_CACHE_NEXT_LINE_PREFETCHER: usize = 17;

/// feature_disable register
#[derive(Clone, Copy, Debug)]
pub struct FeatureDisable {
    bits: usize,
}

impl FeatureDisable {
    /// Returns the contents of the register as raw bits
    #[inline]
    pub fn bits(&self) -> usize {
        self.bits
    }

    /// Disable data cache clock gating
    #[inline]
    pub fn disable_data_cache_clock_gating(&self) -> bool {
        self.bits & (1 << DISABLE_DATA_CACHE_CLOCK_GATING) != 0
    }

    /// Disable instruction cache clock gating
    #[inline]
    pub fn disable_instruction_cache_clock_gating(&self) -> bool {
        self.bits & (1 << DISABLE_INSTRUCTION_CACHE_CLOCK_GATING) != 0
    }

    /// Disable pipeline clock gating
    #[inline]
    pub fn disable_pipeline_clock_gating(&self) -> bool {
        self.bits & (1 << DISABLE_PIPELINE_CLOCK_GATING) != 0
    }

    /// Disable speculative instruction cache refill
    #[inline]
    pub fn disable_speculative_instruction_cache_refill(&self) -> bool {
        self.bits & (1 << DISABLE_SPECULATIVE_INSTRUCTION_CACHE_REFILL) != 0
    }

    /// Suppress corrupt signal on GrantData messages
    #[inline]
    pub fn suppress_corrupt_signal_on_grantdata_messages(&self) -> bool {
        self.bits & (1 << SUPPRESS_CORRUPT_SIGNAL_ON_GRANTDATA_MESSAGE) != 0
    }

    /// Disable short forward branch optimization
    #[inline]
    pub fn disable_short_forward_branch_optimization(&self) -> bool {
        self.bits & (1 << DISABLE_SHORT_FORWARD_BRANCH_OPTIMIZATION) != 0
    }

    /// Disable instruction cache next-line prefetcher
    #[inline]
    pub fn disable_instruction_cache_next_line_prefetcher(&self) -> bool {
        self.bits & (1 << DISABLE_INSTRUCTION_CACHE_NEXT_LINE_PREFETCHER) != 0
    }
}

read_as_csr!(
    /// Reads the [FeatureDisable] from the platform CSR.
    , FeatureDisable, 0x7c1);

write_as_csr!(
    /// Writes the [FeatureDisable] in-memory value to the platform CSR.
    , FeatureDisable, 0x7c1);

set_csr!(0x7c1);
clear_csr!(0x7c1);

set_clear_csr!(
    /// Disable data cache clock gating
    , set_disable_data_cache_clock_gating, clear_disable_data_cache_clock_gating, 1 << DISABLE_DATA_CACHE_CLOCK_GATING);
set_clear_csr!(
    /// Disable instruction cache clock gating
    , set_disable_instruction_cache_clock_gating, clear_disable_instruction_cache_clock_gating, 1 << DISABLE_INSTRUCTION_CACHE_CLOCK_GATING);
set_clear_csr!(
    /// Disable pipeline clock gating
    , set_disable_pipeline_clock_gating, clear_disable_pipeline_clock_gating, 1 << DISABLE_PIPELINE_CLOCK_GATING);
set_clear_csr!(
    /// Disable speculative instruction cache refill
    , set_disable_speculative_instruction_cache_refill, clear_disable_speculative_instruction_cache_refill, 1 << DISABLE_SPECULATIVE_INSTRUCTION_CACHE_REFILL);
set_clear_csr!(
    /// Suppress corrupt signal on GrantData messages
    , set_suppress_corrupt_signal_on_grantdata_messages, clear_suppress_corrupt_signal_on_grantdata_messages, 1 << SUPPRESS_CORRUPT_SIGNAL_ON_GRANTDATA_MESSAGE);
set_clear_csr!(
    /// Disable short forward branch optimization
    , set_disable_short_forward_branch_optimization, clear_disable_short_forward_branch_optimization, 1 << DISABLE_SHORT_FORWARD_BRANCH_OPTIMIZATION);
set_clear_csr!(
    /// Disable instruction cache next-line prefetcher
    , set_disable_instruction_cache_next_line_prefetcher, clear_disable_instruction_cache_next_line_prefetcher, 1 << DISABLE_INSTRUCTION_CACHE_NEXT_LINE_PREFETCHER);
set_clear_csr!(
    /// Disable all features.
    , set_all, clear_all, FIELD_MASK);
