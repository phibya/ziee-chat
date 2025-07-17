//! Quantization support for Candle models
//!
//! This module provides quantization support for running large language models
//! on consumer hardware with reduced memory requirements.

pub mod gptq;

pub use gptq::*;

/// Quantization method types
#[derive(Debug, Clone, PartialEq)]
pub enum QuantMethod {
    /// No quantization (FP16/F32)
    None,
    /// GPTQ 4-bit quantization
    Gptq,
    /// AWQ quantization (compatible with GPTQ)
    Awq,
    /// Marlin optimized format
    Marlin,
    /// GGUF quantization formats
    Gguf,
}

impl std::str::FromStr for QuantMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" | "false" | "disabled" => Ok(QuantMethod::None),
            "gptq" => Ok(QuantMethod::Gptq),
            "awq" => Ok(QuantMethod::Awq),
            "marlin" => Ok(QuantMethod::Marlin),
            "gguf" => Ok(QuantMethod::Gguf),
            "auto" => Ok(QuantMethod::None), // Auto-detect based on model
            _ => Err(format!("Unknown quantization method: {}", s)),
        }
    }
}

impl Default for QuantMethod {
    fn default() -> Self {
        QuantMethod::None
    }
}

/// General quantization configuration
#[derive(Debug, Clone)]
pub struct QuantConfig {
    /// Quantization method
    pub method: QuantMethod,
    /// Number of bits for quantization (4 or 8)
    pub bits: u32,
    /// Group size for group-wise quantization (-1 for per-channel)
    pub group_size: i32,
    /// Symmetric quantization (no zero points)
    pub symmetric: bool,
    /// Activation ordering for GPTQ
    pub desc_act: bool,
    /// Checkpoint format (e.g., "marlin")
    pub checkpoint_format: Option<String>,
}

impl Default for QuantConfig {
    fn default() -> Self {
        Self {
            method: QuantMethod::None,
            bits: 16,
            group_size: 128,
            symmetric: true,
            desc_act: false,
            checkpoint_format: None,
        }
    }
}

impl QuantConfig {
    /// Create a new GPTQ configuration
    pub fn gptq(bits: u32, group_size: i32, symmetric: bool) -> Self {
        Self {
            method: QuantMethod::Gptq,
            bits,
            group_size,
            symmetric,
            desc_act: false,
            checkpoint_format: None,
        }
    }

    /// Create a Marlin-optimized GPTQ configuration
    pub fn marlin(bits: u32, group_size: i32, symmetric: bool) -> Self {
        Self {
            method: QuantMethod::Marlin,
            bits,
            group_size,
            symmetric,
            desc_act: false,
            checkpoint_format: Some("marlin".to_string()),
        }
    }

    /// Check if this configuration is compatible with Marlin optimization
    pub fn is_marlin_compatible(&self) -> bool {
        matches!(self.method, QuantMethod::Gptq | QuantMethod::Awq | QuantMethod::Marlin)
            && (self.bits == 4 || self.bits == 8)
            && (self.group_size == 64 || self.group_size == 128 || self.group_size == -1)
            && self.symmetric
            && !self.desc_act
    }

    /// Get the packing factor (weights per packed element)
    pub fn pack_factor(&self) -> u32 {
        match self.bits {
            4 => 8,  // 8 weights per uint32
            8 => 4,  // 4 weights per uint32
            _ => 1,  // No packing for other bit widths
        }
    }

    /// Validate the quantization configuration
    pub fn validate(&self) -> Result<(), String> {
        match self.method {
            QuantMethod::None => Ok(()),
            QuantMethod::Gptq | QuantMethod::Awq | QuantMethod::Marlin => {
                if self.bits != 4 && self.bits != 8 {
                    return Err(format!("GPTQ only supports 4 or 8 bits, got {}", self.bits));
                }
                if self.group_size != -1 && self.group_size <= 0 {
                    return Err(format!("Invalid group size: {}", self.group_size));
                }
                Ok(())
            }
            QuantMethod::Gguf => {
                // GGUF validation will be added later
                Ok(())
            }
        }
    }
}