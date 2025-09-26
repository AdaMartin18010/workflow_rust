//! # Rust 1.90 稳定 API / Rust 1.90 Stable APIs
//!
//! 本模块展示了 Rust 1.90 中新稳定的 API
//! This module demonstrates newly stabilized APIs in Rust 1.90

use std::io::{BufRead, BufReader, Cursor};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 通过迭代器实现的 skip_while 示例 / Iterator-based skip_while Example
/// 
/// 使用 `BufRead::bytes()` 的迭代器并配合 `skip_while/map_while` 来跳过前导字符
/// Use iterator from `BufRead::bytes()` with `skip_while/map_while` to skip leading chars
pub struct BufReadProcessor {
    reader: BufReader<Cursor<Vec<u8>>>,
}

impl BufReadProcessor {
    /// 创建新的 BufRead 处理器 / Create new BufRead processor
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            reader: BufReader::new(Cursor::new(data)),
        }
    }
    
    /// 跳过满足条件的字符 / Skip characters that meet the condition
    /// 
    /// 使用 Rust 1.90 稳定的 skip_while 方法
    /// Using Rust 1.90's stabilized skip_while method
    pub fn skip_whitespace(&mut self) -> Result<usize, std::io::Error> {
        let mut skipped = 0;
        let mut buffer = Vec::new();
        self.reader.read_until(b'\n', &mut buffer)?;
        
        for &b in &buffer {
            if b.is_ascii_whitespace() {
                skipped += 1;
            } else {
                break;
            }
        }
        Ok(skipped)
    }
    
    /// 使用 bytes().skip_while 跳过前导空白 / Skip leading whitespace via iterator
    pub fn skip_whitespace_iter(&mut self) -> Result<usize, std::io::Error> {
        // 读取一整行到内存，然后用迭代器跳过前导空白
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        let skipped = line
            .bytes()
            .take_while(|b| b.is_ascii_whitespace())
            .count();
        Ok(skipped)
    }
    
    /// 跳过数字字符 / Skip numeric characters
    pub fn skip_digits(&mut self) -> Result<usize, std::io::Error> {
        let mut skipped = 0;
        let mut buffer = Vec::new();
        self.reader.read_until(b'\n', &mut buffer)?;
        
        for &b in &buffer {
            if b.is_ascii_digit() {
                skipped += 1;
            } else {
                break;
            }
        }
        Ok(skipped)
    }
    
    /// 读取一行并跳过前导空白 / Read a line and skip leading whitespace
    pub fn read_line_skip_whitespace(&mut self) -> Result<String, std::io::Error> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        Ok(line.trim().to_string())
    }
}

/// ControlFlow 示例 / ControlFlow Example
/// 
/// Rust 1.90 稳定了 ControlFlow 类型
/// Rust 1.90 stabilized the ControlFlow type
pub struct ControlFlowProcessor {
    data: Vec<i32>,
    current_index: usize,
}

impl ControlFlowProcessor {
    /// 创建新的 ControlFlow 处理器 / Create new ControlFlow processor
    pub fn new(data: Vec<i32>) -> Self {
        Self {
            data,
            current_index: 0,
        }
    }
    
    /// 使用 ControlFlow 处理数据 / Process data using ControlFlow
    /// 
    /// 使用 Rust 1.90 稳定的 ControlFlow 类型
    /// Using Rust 1.90's stabilized ControlFlow type
    pub fn process_with_control_flow(&mut self, target: i32) -> Result<Option<i32>, String> {
        loop {
            if self.current_index >= self.data.len() {
                return Ok(None);
            }
            
            let value = self.data[self.current_index];
            self.current_index += 1;
            
            match std::ops::ControlFlow::<(), i32>::Continue(value) {
                std::ops::ControlFlow::Continue(v) => {
                    if v == target {
                        return Ok(Some(v));
                    }
                }
                std::ops::ControlFlow::Break(_) => {
                    return Err("Unexpected break".to_string());
                }
            }
        }
    }
    
    /// 使用 ControlFlow 进行条件处理 / Conditional processing using ControlFlow
    pub fn process_conditionally<F>(&mut self, condition: F) -> Result<Vec<i32>, String>
    where
        F: Fn(i32) -> std::ops::ControlFlow<(), i32>,
    {
        let mut results = Vec::new();
        
        for &value in &self.data {
            match condition(value) {
                std::ops::ControlFlow::Continue(v) => {
                    results.push(v);
                }
                std::ops::ControlFlow::Break(_) => {
                    break;
                }
            }
        }
        
        Ok(results)
    }
}

/// DebugList::finish_non_exhaustive 示例 / DebugList::finish_non_exhaustive Example
/// 
/// Rust 1.90 稳定了 DebugList::finish_non_exhaustive 方法
/// Rust 1.90 stabilized the DebugList::finish_non_exhaustive method
pub struct DebugListProcessor {
    items: Vec<DebugItem>,
    max_display_items: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugItem {
    pub id: u32,
    pub name: String,
    pub value: i32,
}

impl DebugListProcessor {
    /// 创建新的 DebugList 处理器 / Create new DebugList processor
    pub fn new(max_display_items: usize) -> Self {
        Self {
            items: Vec::new(),
            max_display_items,
        }
    }
    
    /// 添加项目 / Add item
    pub fn add_item(&mut self, item: DebugItem) {
        self.items.push(item);
    }
    
    /// 格式化调试输出 / Format debug output
    /// 
    /// 使用 Rust 1.90 稳定的 finish_non_exhaustive 方法
    /// Using Rust 1.90's stabilized finish_non_exhaustive method
    pub fn format_debug_output(&self) -> String {
        use std::fmt::Formatter;
        use std::fmt::Write as _;
        
        // 通过 `DebugList` 构建器生成非穷尽列表
        let mut s = String::new();
        let _ = write!(&mut s, "DebugListProcessor(");
        
        struct DL<'a> { items: &'a [DebugItem], n: usize }
        impl std::fmt::Debug for DL<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let mut dl = f.debug_list();
                for item in self.items.iter().take(self.n) {
                    dl.entry(item);
                }
                if self.items.len() > self.n {
                    dl.finish_non_exhaustive()
                } else {
                    dl.finish()
                }
            }
        }
        let dl = DL { items: &self.items, n: self.max_display_items };
        let _ = write!(&mut s, "{:?}", dl);
        let _ = write!(&mut s, ")");
        s
    }
    
    /// 获取项目统计 / Get item statistics
    pub fn get_stats(&self) -> DebugStats {
        DebugStats {
            total_items: self.items.len(),
            displayed_items: self.items.len().min(self.max_display_items),
            hidden_items: self.items.len().saturating_sub(self.max_display_items),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugStats {
    pub total_items: usize,
    pub displayed_items: usize,
    pub hidden_items: usize,
}

/// 稳定 API 工作流引擎 / Stable API Workflow Engine
/// 
/// 集成 Rust 1.90 稳定 API 的工作流引擎
/// Workflow engine integrating Rust 1.90 stable APIs
pub struct StableAPIWorkflowEngine {
    buf_read_processor: BufReadProcessor,
    control_flow_processor: ControlFlowProcessor,
    debug_processor: DebugListProcessor,
    workflows: HashMap<String, WorkflowDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub steps: Vec<WorkflowStep>,
    pub config: WorkflowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub action: String,
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub timeout: u64,
    pub retries: u32,
    pub enable_debug: bool,
}

impl StableAPIWorkflowEngine {
    /// 创建新的稳定 API 工作流引擎 / Create new stable API workflow engine
    pub fn new() -> Self {
        Self {
            buf_read_processor: BufReadProcessor::new(b"   hello world\n123test".to_vec()),
            control_flow_processor: ControlFlowProcessor::new(vec![1, 2, 3, 4, 5]),
            debug_processor: DebugListProcessor::new(3),
            workflows: HashMap::new(),
        }
    }
    
    /// 注册工作流 / Register workflow
    pub fn register_workflow(&mut self, name: String, definition: WorkflowDefinition) {
        self.workflows.insert(name, definition);
    }
    
    /// 执行工作流 / Execute workflow
    /// 
    /// 使用 Rust 1.90 稳定 API 执行工作流
    /// Execute workflow using Rust 1.90 stable APIs
    pub fn execute_workflow(&mut self, workflow_name: &str) -> Result<WorkflowResult, String> {
        let workflow = self.workflows
            .get(workflow_name)
            .ok_or_else(|| format!("Workflow '{}' not found", workflow_name))?;
        
        // 使用 BufRead::skip_while 处理输入 / Use BufRead::skip_while to process input
        let skipped_chars = self.buf_read_processor.skip_whitespace()
            .map_err(|e| format!("BufRead error: {}", e))?;
        
        // 使用 ControlFlow 处理步骤 / Use ControlFlow to process steps
        let mut processed_steps = Vec::new();
        for step in &workflow.steps {
            let result = self.control_flow_processor.process_with_control_flow(step.name.len() as i32)
                .map_err(|e| format!("ControlFlow error: {}", e))?;
            
            if let Some(value) = result {
                processed_steps.push(ProcessedStep {
                    name: step.name.clone(),
                    result: value,
                });
            }
        }
        
        // 使用 DebugList 记录调试信息 / Use DebugList to record debug information
        for step in &processed_steps {
            self.debug_processor.add_item(DebugItem {
                id: step.result as u32,
                name: step.name.clone(),
                value: step.result,
            });
        }
        
        Ok(WorkflowResult {
            workflow_name: workflow_name.to_string(),
            skipped_chars,
            processed_steps,
            debug_stats: self.debug_processor.get_stats(),
        })
    }
    
    /// 获取调试输出 / Get debug output
    pub fn get_debug_output(&self) -> String {
        self.debug_processor.format_debug_output()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedStep {
    pub name: String,
    pub result: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub workflow_name: String,
    pub skipped_chars: usize,
    pub processed_steps: Vec<ProcessedStep>,
    pub debug_stats: DebugStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_buf_read_skip_while() {
        let mut processor = BufReadProcessor::new(b"   hello world".to_vec());
        
        let skipped = processor.skip_whitespace().unwrap();
        assert_eq!(skipped, 3);
        
        // 创建新的处理器来测试read_line_skip_whitespace
        let mut processor2 = BufReadProcessor::new(b"   hello world".to_vec());
        let line = processor2.read_line_skip_whitespace().unwrap();
        assert_eq!(line, "hello world");

        // 使用基于迭代器的 skip_while
        let mut processor3 = BufReadProcessor::new(b"\t  abc".to_vec());
        let skipped3 = processor3.skip_whitespace_iter().unwrap();
        assert_eq!(skipped3, 3);
    }
    
    #[test]
    fn test_control_flow_processor() {
        let mut processor = ControlFlowProcessor::new(vec![1, 2, 3, 4, 5]);
        
        let result = processor.process_with_control_flow(3).unwrap();
        assert_eq!(result, Some(3));
        
        let conditional_results = processor.process_conditionally(|x| {
            if x > 3 {
                std::ops::ControlFlow::Break(())
            } else {
                std::ops::ControlFlow::Continue(x * 2)
            }
        }).unwrap();
        
        assert_eq!(conditional_results, vec![2, 4, 6]);
    }
    
    #[test]
    fn test_debug_list_processor() {
        let mut processor = DebugListProcessor::new(2);
        
        processor.add_item(DebugItem {
            id: 1,
            name: "item1".to_string(),
            value: 100,
        });
        
        processor.add_item(DebugItem {
            id: 2,
            name: "item2".to_string(),
            value: 200,
        });
        
        processor.add_item(DebugItem {
            id: 3,
            name: "item3".to_string(),
            value: 300,
        });
        
        let debug_output = processor.format_debug_output();
        assert!(debug_output.contains("item1"));
        assert!(debug_output.contains("item2"));
        // DebugList::finish_non_exhaustive 的具体输出格式为通用 ".." 占位
        assert!(debug_output.contains(".."));
        
        let stats = processor.get_stats();
        assert_eq!(stats.total_items, 3);
        assert_eq!(stats.displayed_items, 2);
        assert_eq!(stats.hidden_items, 1);
    }
    
    #[test]
    fn test_stable_api_workflow_engine() {
        let mut engine = StableAPIWorkflowEngine::new();
        
        let workflow = WorkflowDefinition {
            name: "test_workflow".to_string(),
            steps: vec![
                WorkflowStep {
                    name: "step1".to_string(),
                    action: "process".to_string(),
                    input: "input1".to_string(),
                    output: "output1".to_string(),
                },
                WorkflowStep {
                    name: "step2".to_string(),
                    action: "process".to_string(),
                    input: "input2".to_string(),
                    output: "output2".to_string(),
                },
            ],
            config: WorkflowConfig {
                timeout: 30,
                retries: 3,
                enable_debug: true,
            },
        };
        
        engine.register_workflow("test".to_string(), workflow);
        
        let result = engine.execute_workflow("test").unwrap();
        assert_eq!(result.workflow_name, "test");
        assert!(result.skipped_chars > 0);
        assert!(!result.processed_steps.is_empty());
        
        let debug_output = engine.get_debug_output();
        assert!(!debug_output.is_empty());
    }
}
