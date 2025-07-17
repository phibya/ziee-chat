use super::super::candle::CandleError;
use candle_core::{Device, Tensor};
use candle_transformers::models::llama::Cache;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use uuid::Uuid;

/// Sequence status in the scheduler
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SequenceStatus {
    /// Sequence is waiting to be processed
    Waiting,
    /// Sequence is currently being processed
    Running,
    /// Sequence has been swapped out to CPU memory
    SwappedOut,
    /// Sequence has finished processing
    Finished,
    /// Sequence was aborted due to error
    Aborted,
}

/// Preemption mode for handling resource constraints
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PreemptionMode {
    /// Recompute the sequence from the beginning
    Recompute,
    /// Swap to CPU memory
    Swap,
}

/// A single sequence in the scheduler
#[derive(Debug)]
pub struct Sequence {
    pub id: Uuid,
    pub prompt: String,
    pub input_ids: Vec<u32>,
    pub generated_ids: Vec<u32>,
    pub status: SequenceStatus,
    pub created_at: std::time::Instant,
    pub last_token_time: std::time::Instant,
    pub max_tokens: Option<usize>,
    pub temperature: f64,
    pub top_p: f64,
    pub logical_blocks: Vec<LogicalBlock>,
    pub physical_blocks: Vec<PhysicalBlock>,
    pub cache: Option<Cache>,
    pub response_tx: Option<oneshot::Sender<Result<String, CandleError>>>,
}

/// Logical block representing a sequence of tokens
#[derive(Debug, Clone)]
pub struct LogicalBlock {
    pub id: Uuid,
    pub tokens: Vec<u32>,
    pub is_full: bool,
    pub physical_block_id: Option<Uuid>,
}

/// Physical block in memory
#[derive(Debug, Clone)]
pub struct PhysicalBlock {
    pub id: Uuid,
    pub device: Device,
    pub ref_count: usize,
    pub data: Option<Tensor>,
}

/// Group of sequences that can be batched together
#[derive(Debug)]
pub struct SequenceGroup {
    pub id: Uuid,
    pub sequences: Vec<Sequence>,
    pub created_at: std::time::Instant,
    pub priority: i32,
}

/// Block manager for memory allocation
#[derive(Debug)]
pub struct BlockManager {
    pub block_size: usize,
    pub num_gpu_blocks: usize,
    pub num_cpu_blocks: usize,
    pub gpu_allocator: BlockAllocator,
    pub cpu_allocator: BlockAllocator,
    pub block_tables: HashMap<Uuid, Vec<PhysicalBlock>>,
}

/// Block allocator for a specific device
#[derive(Debug)]
pub struct BlockAllocator {
    pub device: Device,
    pub num_blocks: usize,
    pub free_blocks: VecDeque<PhysicalBlock>,
    pub allocated_blocks: HashMap<Uuid, PhysicalBlock>,
}

/// Scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub max_num_seqs: usize,
    pub block_size: usize,
    pub max_model_len: usize,
    pub max_num_batched_tokens: usize,
    pub max_paddings: usize,
    pub preemption_mode: PreemptionMode,
    pub enable_paged_attention: bool,
}

/// Output from the scheduler
#[derive(Debug)]
pub struct SchedulerOutput {
    pub scheduled_groups: Vec<SequenceGroup>,
    pub preempted_groups: Vec<SequenceGroup>,
    pub ignored_groups: Vec<SequenceGroup>,
    pub blocks_to_swap_in: HashMap<Uuid, Vec<PhysicalBlock>>,
    pub blocks_to_swap_out: HashMap<Uuid, Vec<PhysicalBlock>>,
    pub blocks_to_copy: HashMap<Uuid, Vec<PhysicalBlock>>,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_num_seqs: 256,
            block_size: 32,
            max_model_len: 2048,
            max_num_batched_tokens: 2048,
            max_paddings: 256,
            preemption_mode: PreemptionMode::Recompute,
            enable_paged_attention: false,
        }
    }
}

impl LogicalBlock {
    pub fn new(tokens: Vec<u32>, block_size: usize) -> Self {
        let is_full = tokens.len() >= block_size;
        Self {
            id: Uuid::new_v4(),
            tokens,
            is_full,
            physical_block_id: None,
        }
    }

    pub fn append_token(&mut self, token: u32, block_size: usize) {
        self.tokens.push(token);
        self.is_full = self.tokens.len() >= block_size;
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn num_tokens(&self) -> usize {
        self.tokens.len()
    }
}

impl PhysicalBlock {
    pub fn new(device: Device) -> Self {
        Self {
            id: Uuid::new_v4(),
            device,
            ref_count: 0,
            data: None,
        }
    }

    pub fn increment_ref(&mut self) {
        self.ref_count += 1;
    }

    pub fn decrement_ref(&mut self) {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
    }

    pub fn is_free(&self) -> bool {
        self.ref_count == 0
    }
}

impl Clone for Sequence {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            prompt: self.prompt.clone(),
            input_ids: self.input_ids.clone(),
            generated_ids: self.generated_ids.clone(),
            status: self.status.clone(),
            created_at: self.created_at,
            last_token_time: self.last_token_time,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            top_p: self.top_p,
            logical_blocks: self.logical_blocks.clone(),
            physical_blocks: self.physical_blocks.clone(),
            cache: self.cache.clone(),
            response_tx: None, // Cannot clone oneshot::Sender
        }
    }
}

impl Clone for SequenceGroup {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            sequences: self.sequences.clone(),
            created_at: self.created_at,
            priority: self.priority,
        }
    }
}

impl Sequence {
    pub fn new(
        prompt: String,
        input_ids: Vec<u32>,
        max_tokens: Option<usize>,
        temperature: f64,
        top_p: f64,
        response_tx: oneshot::Sender<Result<String, CandleError>>,
    ) -> Self {
        let now = std::time::Instant::now();
        Self {
            id: Uuid::new_v4(),
            prompt,
            input_ids,
            generated_ids: Vec::new(),
            status: SequenceStatus::Waiting,
            created_at: now,
            last_token_time: now,
            max_tokens,
            temperature,
            top_p,
            logical_blocks: Vec::new(),
            physical_blocks: Vec::new(),
            cache: None,
            response_tx: Some(response_tx),
        }
    }

    pub fn get_len(&self) -> usize {
        self.input_ids.len() + self.generated_ids.len()
    }

    pub fn get_prompt_len(&self) -> usize {
        self.input_ids.len()
    }

    pub fn get_output_len(&self) -> usize {
        self.generated_ids.len()
    }

    pub fn is_finished(&self) -> bool {
        if let Some(max_tokens) = self.max_tokens {
            self.get_output_len() >= max_tokens
        } else {
            false
        }
    }

    pub fn append_token(&mut self, token: u32) {
        self.generated_ids.push(token);
        self.last_token_time = std::time::Instant::now();
    }

    pub fn get_num_uncomputed_tokens(&self) -> usize {
        // Return the number of tokens that haven't been computed yet
        let total_tokens = self.get_len();
        let computed_tokens = self.logical_blocks.iter().map(|b| b.num_tokens()).sum::<usize>();
        total_tokens.saturating_sub(computed_tokens)
    }
}

impl SequenceGroup {
    pub fn new(sequences: Vec<Sequence>, priority: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            sequences,
            created_at: std::time::Instant::now(),
            priority,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.sequences.iter().all(|seq| seq.is_finished())
    }

    pub fn get_max_num_running_seqs(&self) -> usize {
        self.sequences.len()
    }

    pub fn get_seqs(&self, status: Option<SequenceStatus>) -> Vec<&Sequence> {
        match status {
            Some(status) => self.sequences.iter().filter(|seq| seq.status == status).collect(),
            None => self.sequences.iter().collect(),
        }
    }

    pub fn num_seqs(&self, status: Option<SequenceStatus>) -> usize {
        self.get_seqs(status).len()
    }
}

impl BlockAllocator {
    pub fn new(device: Device, num_blocks: usize, block_size: usize) -> Self {
        let mut free_blocks = VecDeque::new();
        
        // Initialize free blocks
        for _ in 0..num_blocks {
            free_blocks.push_back(PhysicalBlock::new(device.clone()));
        }
        
        Self {
            device,
            num_blocks,
            free_blocks,
            allocated_blocks: HashMap::new(),
        }
    }

    pub fn allocate(&mut self) -> Option<PhysicalBlock> {
        if let Some(mut block) = self.free_blocks.pop_front() {
            block.increment_ref();
            let block_id = block.id;
            self.allocated_blocks.insert(block_id, block.clone());
            Some(block)
        } else {
            None
        }
    }

    pub fn free(&mut self, block_id: Uuid) -> bool {
        if let Some(mut block) = self.allocated_blocks.remove(&block_id) {
            block.decrement_ref();
            if block.is_free() {
                self.free_blocks.push_back(block);
                true
            } else {
                // Block still has references, put it back
                self.allocated_blocks.insert(block_id, block);
                false
            }
        } else {
            false
        }
    }

    pub fn get_num_free_blocks(&self) -> usize {
        self.free_blocks.len()
    }

    pub fn get_num_allocated_blocks(&self) -> usize {
        self.allocated_blocks.len()
    }
}

impl BlockManager {
    pub fn new(
        block_size: usize,
        num_gpu_blocks: usize,
        num_cpu_blocks: usize,
        gpu_device: Device,
        cpu_device: Device,
    ) -> Self {
        let gpu_allocator = BlockAllocator::new(gpu_device, num_gpu_blocks, block_size);
        let cpu_allocator = BlockAllocator::new(cpu_device, num_cpu_blocks, block_size);
        
        Self {
            block_size,
            num_gpu_blocks,
            num_cpu_blocks,
            gpu_allocator,
            cpu_allocator,
            block_tables: HashMap::new(),
        }
    }

    pub fn can_allocate(&self, seq_group: &SequenceGroup) -> bool {
        // Check if we have enough free blocks to allocate for this sequence group
        let required_blocks = self.get_num_required_blocks(seq_group);
        self.gpu_allocator.get_num_free_blocks() >= required_blocks
    }

    pub fn allocate(&mut self, seq_group: &SequenceGroup) -> Vec<PhysicalBlock> {
        let mut allocated_blocks = Vec::new();
        let required_blocks = self.get_num_required_blocks(seq_group);
        
        for _ in 0..required_blocks {
            if let Some(block) = self.gpu_allocator.allocate() {
                allocated_blocks.push(block);
            } else {
                // Allocation failed, free any allocated blocks
                for block in allocated_blocks.iter() {
                    self.gpu_allocator.free(block.id);
                }
                return Vec::new();
            }
        }
        
        // Store block table mapping
        for seq in &seq_group.sequences {
            self.block_tables.insert(seq.id, allocated_blocks.clone());
        }
        
        allocated_blocks
    }

    pub fn free(&mut self, seq_id: Uuid) {
        if let Some(blocks) = self.block_tables.remove(&seq_id) {
            for block in blocks {
                self.gpu_allocator.free(block.id);
            }
        }
    }

    pub fn get_num_required_blocks(&self, seq_group: &SequenceGroup) -> usize {
        // Calculate the number of blocks required for this sequence group
        let mut total_tokens = 0;
        for seq in &seq_group.sequences {
            total_tokens += seq.get_len();
        }
        
        // Calculate blocks needed (rounded up)
        (total_tokens + self.block_size - 1) / self.block_size
    }

    pub fn get_num_free_gpu_blocks(&self) -> usize {
        self.gpu_allocator.get_num_free_blocks()
    }

    pub fn get_num_free_cpu_blocks(&self) -> usize {
        self.cpu_allocator.get_num_free_blocks()
    }

    pub fn swap_out(&mut self, seq_id: Uuid) -> Vec<PhysicalBlock> {
        // Implementation for swapping blocks from GPU to CPU
        if let Some(gpu_blocks) = self.block_tables.get(&seq_id) {
            let mut cpu_blocks = Vec::new();
            
            for gpu_block in gpu_blocks {
                if let Some(cpu_block) = self.cpu_allocator.allocate() {
                    // Copy data from GPU to CPU block
                    // This would involve actual tensor copying in a real implementation
                    cpu_blocks.push(cpu_block);
                }
            }
            
            // Free GPU blocks
            for gpu_block in gpu_blocks {
                self.gpu_allocator.free(gpu_block.id);
            }
            
            // Update block table with CPU blocks
            self.block_tables.insert(seq_id, cpu_blocks.clone());
            
            cpu_blocks
        } else {
            Vec::new()
        }
    }

    pub fn swap_in(&mut self, seq_id: Uuid) -> Vec<PhysicalBlock> {
        // Implementation for swapping blocks from CPU to GPU
        if let Some(cpu_blocks) = self.block_tables.get(&seq_id) {
            let mut gpu_blocks = Vec::new();
            
            for cpu_block in cpu_blocks {
                if let Some(gpu_block) = self.gpu_allocator.allocate() {
                    // Copy data from CPU to GPU block
                    // This would involve actual tensor copying in a real implementation
                    gpu_blocks.push(gpu_block);
                }
            }
            
            // Free CPU blocks
            for cpu_block in cpu_blocks {
                self.cpu_allocator.free(cpu_block.id);
            }
            
            // Update block table with GPU blocks
            self.block_tables.insert(seq_id, gpu_blocks.clone());
            
            gpu_blocks
        } else {
            Vec::new()
        }
    }
}

/// Advanced scheduler implementing candle-vLLM style scheduling
pub struct Scheduler {
    config: SchedulerConfig,
    block_manager: Arc<Mutex<BlockManager>>,
    waiting: VecDeque<SequenceGroup>,
    running: Vec<SequenceGroup>,
    swapped: Vec<SequenceGroup>,
}

impl Scheduler {
    pub fn new(config: SchedulerConfig, block_manager: Arc<Mutex<BlockManager>>) -> Self {
        Self {
            config,
            block_manager,
            waiting: VecDeque::new(),
            running: Vec::new(),
            swapped: Vec::new(),
        }
    }

    pub async fn add_sequence_group(&mut self, seq_group: SequenceGroup) {
        self.waiting.push_back(seq_group);
    }

    pub async fn schedule(&mut self) -> SchedulerOutput {
        let mut scheduled_groups = Vec::new();
        let mut preempted_groups = Vec::new();
        let mut ignored_groups = Vec::new();
        let mut blocks_to_swap_in = HashMap::new();
        let mut blocks_to_swap_out = HashMap::new();
        let blocks_to_copy = HashMap::new();

        // Schedule swapped sequences first
        self.schedule_swapped(&mut scheduled_groups, &mut blocks_to_swap_in).await;
        
        // Schedule running sequences
        self.schedule_running(&mut scheduled_groups, &mut preempted_groups, &mut blocks_to_swap_out).await;
        
        // Schedule waiting sequences
        self.schedule_waiting(&mut scheduled_groups, &mut ignored_groups).await;

        SchedulerOutput {
            scheduled_groups,
            preempted_groups,
            ignored_groups,
            blocks_to_swap_in,
            blocks_to_swap_out,
            blocks_to_copy,
        }
    }

    async fn schedule_swapped(
        &mut self,
        scheduled_groups: &mut Vec<SequenceGroup>,
        blocks_to_swap_in: &mut HashMap<Uuid, Vec<PhysicalBlock>>,
    ) {
        let mut block_manager = self.block_manager.lock().await;
        let mut i = 0;
        
        while i < self.swapped.len() {
            let seq_group = &self.swapped[i];
            
            // Check if we have enough GPU blocks to swap in
            if block_manager.can_allocate(seq_group) {
                let seq_group = self.swapped.remove(i);
                
                // Swap blocks from CPU to GPU
                let gpu_blocks = block_manager.swap_in(seq_group.id);
                blocks_to_swap_in.insert(seq_group.id, gpu_blocks);
                
                self.running.push(seq_group.clone());
                scheduled_groups.push(seq_group);
            } else {
                i += 1;
            }
        }
    }

    async fn schedule_running(
        &mut self,
        scheduled_groups: &mut Vec<SequenceGroup>,
        preempted_groups: &mut Vec<SequenceGroup>,
        blocks_to_swap_out: &mut HashMap<Uuid, Vec<PhysicalBlock>>,
    ) {
        let mut block_manager = self.block_manager.lock().await;
        let mut i = 0;
        
        while i < self.running.len() {
            let seq_group = &self.running[i];
            
            // Check if sequence group is finished
            if seq_group.is_finished() {
                let seq_group = self.running.remove(i);
                
                // Free blocks for finished sequences
                for seq in &seq_group.sequences {
                    block_manager.free(seq.id);
                }
                continue;
            }
            
            // Check if we need to preempt this sequence group
            if self.should_preempt(seq_group, &*block_manager).await {
                let seq_group = self.running.remove(i);
                
                match self.config.preemption_mode {
                    PreemptionMode::Recompute => {
                        // Free blocks and move to waiting queue
                        for seq in &seq_group.sequences {
                            block_manager.free(seq.id);
                        }
                        self.waiting.push_back(seq_group.clone());
                    }
                    PreemptionMode::Swap => {
                        // Swap blocks to CPU
                        let cpu_blocks = block_manager.swap_out(seq_group.id);
                        blocks_to_swap_out.insert(seq_group.id, cpu_blocks);
                        self.swapped.push(seq_group.clone());
                    }
                }
                
                preempted_groups.push(seq_group);
            } else {
                scheduled_groups.push(seq_group.clone());
                i += 1;
            }
        }
    }

    async fn schedule_waiting(
        &mut self,
        scheduled_groups: &mut Vec<SequenceGroup>,
        ignored_groups: &mut Vec<SequenceGroup>,
    ) {
        let mut block_manager = self.block_manager.lock().await;
        
        while let Some(seq_group) = self.waiting.pop_front() {
            // Check if we can allocate blocks for this sequence group
            if block_manager.can_allocate(&seq_group) && 
               self.running.len() < self.config.max_num_seqs {
                
                // Allocate blocks
                let blocks = block_manager.allocate(&seq_group);
                if !blocks.is_empty() {
                    self.running.push(seq_group.clone());
                    scheduled_groups.push(seq_group);
                } else {
                    // Allocation failed, put back in waiting queue
                    self.waiting.push_front(seq_group);
                    break;
                }
            } else {
                // Cannot allocate, ignore for now
                ignored_groups.push(seq_group.clone());
                self.waiting.push_front(seq_group);
                break;
            }
        }
    }

    async fn should_preempt(&self, seq_group: &SequenceGroup, block_manager: &BlockManager) -> bool {
        // Simple preemption logic: preempt if we're running out of blocks
        // and there are waiting sequences
        if self.waiting.is_empty() {
            return false;
        }
        
        let free_blocks = block_manager.get_num_free_gpu_blocks();
        let required_blocks = block_manager.get_num_required_blocks(seq_group);
        
        // Preempt if we have less than 25% of blocks free
        let total_blocks = block_manager.num_gpu_blocks;
        let min_free_blocks = total_blocks / 4;
        
        free_blocks < min_free_blocks && required_blocks > 0
    }

    pub fn get_num_unfinished_seqs(&self) -> usize {
        self.waiting.len() + self.running.len() + self.swapped.len()
    }

    pub fn has_unfinished_seqs(&self) -> bool {
        self.get_num_unfinished_seqs() > 0
    }

    pub fn get_num_waiting_seqs(&self) -> usize {
        self.waiting.len()
    }

    pub fn get_num_running_seqs(&self) -> usize {
        self.running.len()
    }

    pub fn get_num_swapped_seqs(&self) -> usize {
        self.swapped.len()
    }
}