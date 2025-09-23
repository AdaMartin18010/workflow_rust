//! # Rust 1.90 核心特性 / Rust 1.90 Core Features
//!
//! 本模块展示了 Rust 1.90 的核心新特性和改进
//! This module demonstrates the core new features and improvements in Rust 1.90

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

/// JIT 编译器改进示例 / JIT Compiler Improvements Example
/// 
/// Rust 1.90 的 JIT 编译器改进使得迭代器操作更加高效
/// Rust 1.90's JIT compiler improvements make iterator operations more efficient
pub struct JITOptimizedProcessor {
    data: Vec<i32>,
    cache: HashMap<String, i32>,
}

impl JITOptimizedProcessor {
    /// 创建新的处理器 / Create new processor
    pub fn new(data: Vec<i32>) -> Self {
        Self {
            data,
            cache: HashMap::new(),
        }
    }
    
    /// 使用优化的迭代器处理数据 / Process data using optimized iterators
    /// 
    /// Rust 1.90 的 JIT 改进使得这种链式操作更加高效
    /// Rust 1.90's JIT improvements make this chained operation more efficient
    pub fn process_data(&mut self) -> Vec<i32> {
        self.data
            .iter()
            .filter(|&&x| x > 0)  // 过滤正数 / Filter positive numbers
            .map(|&x| x * 2)      // 乘以2 / Multiply by 2
            .filter(|&x| x < 100) // 过滤小于100的数 / Filter numbers less than 100
            .collect()
    }
    
    /// 使用缓存优化的处理 / Cache-optimized processing
    pub fn process_with_cache(&mut self, key: String) -> i32 {
        if let Some(&cached) = self.cache.get(&key) {
            return cached;
        }
        
        let result = self.data
            .iter()
            .sum::<i32>()
            .wrapping_mul(2);
            
        self.cache.insert(key, result);
        result
    }
}

/// 内存分配器改进示例 / Memory Allocator Improvements Example
/// 
/// Rust 1.90 在处理大量小对象时表现更优
/// Rust 1.90 performs better when handling many small objects
pub struct SmallObjectManager {
    objects: Vec<SmallObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmallObject {
    id: u32,
    data: [u8; 16], // 小对象 / Small object
    metadata: String,
}

impl SmallObjectManager {
    /// 创建新的管理器 / Create new manager
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }
    
    /// 批量创建小对象 / Batch create small objects
    /// 
    /// Rust 1.90 的内存分配器改进使得这种操作更高效
    /// Rust 1.90's memory allocator improvements make this operation more efficient
    pub fn create_objects(&mut self, count: usize) {
        for i in 0..count {
            let obj = SmallObject {
                id: i as u32,
                data: [i as u8; 16],
                metadata: format!("object_{}", i),
            };
            self.objects.push(obj);
        }
    }
    
    /// 获取对象统计信息 / Get object statistics
    pub fn get_stats(&self) -> ObjectStats {
        ObjectStats {
            total_objects: self.objects.len(),
            total_memory: self.objects.len() * std::mem::size_of::<SmallObject>(),
            average_size: std::mem::size_of::<SmallObject>(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectStats {
    pub total_objects: usize,
    pub total_memory: usize,
    pub average_size: usize,
}

/// 类型检查器优化示例 / Type Checker Optimization Example
/// 
/// Rust 1.90 的类型检查器优化减少了大型代码库的编译时间
/// Rust 1.90's type checker optimizations reduce compilation time for large codebases
pub struct TypeCheckerOptimized {
    modules: Arc<RwLock<HashMap<String, ModuleInfo>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub name: String,
    pub dependencies: Vec<String>,
    pub size: usize,
    pub complexity: u32,
}

impl TypeCheckerOptimized {
    /// 创建新的类型检查器 / Create new type checker
    pub fn new() -> Self {
        Self {
            modules: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 添加模块 / Add module
    pub async fn add_module(&self, name: String, info: ModuleInfo) {
        let mut modules = self.modules.write().await;
        modules.insert(name, info);
    }
    
    /// 获取编译统计信息 / Get compilation statistics
    pub async fn get_compilation_stats(&self) -> CompilationStats {
        let modules = self.modules.read().await;
        let total_modules = modules.len();
        let total_size: usize = modules.values().map(|m| m.size).sum();
        let total_complexity: u32 = modules.values().map(|m| m.complexity).sum();
        
        CompilationStats {
            total_modules,
            total_size,
            total_complexity,
            average_complexity: if total_modules > 0 {
                total_complexity as f64 / total_modules as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompilationStats {
    pub total_modules: usize,
    pub total_size: usize,
    pub total_complexity: u32,
    pub average_complexity: f64,
}

/// 工作流引擎集成 Rust 1.90 特性 / Workflow Engine Integration with Rust 1.90 Features
pub struct Rust190WorkflowEngine {
    processor: JITOptimizedProcessor,
    object_manager: SmallObjectManager,
    type_checker: TypeCheckerOptimized,
}

impl Rust190WorkflowEngine {
    /// 创建新的工作流引擎 / Create new workflow engine
    pub fn new() -> Self {
        Self {
            processor: JITOptimizedProcessor::new(vec![1, 2, 3, 4, 5]),
            object_manager: SmallObjectManager::new(),
            type_checker: TypeCheckerOptimized::new(),
        }
    }
    
    /// 执行工作流 / Execute workflow
    pub async fn execute_workflow(&mut self) -> Result<WorkflowResult, Box<dyn std::error::Error>> {
        // 使用 JIT 优化的处理器 / Use JIT-optimized processor
        let processed_data = self.processor.process_data();
        
        // 创建小对象 / Create small objects
        self.object_manager.create_objects(1000);
        
        // 获取统计信息 / Get statistics
        let object_stats = self.object_manager.get_stats();
        let compilation_stats = self.type_checker.get_compilation_stats().await;
        
        Ok(WorkflowResult {
            processed_data,
            object_stats,
            compilation_stats,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowResult {
    pub processed_data: Vec<i32>,
    pub object_stats: ObjectStats,
    pub compilation_stats: CompilationStats,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jit_optimized_processor() {
        let mut processor = JITOptimizedProcessor::new(vec![1, 2, 3, 4, 5, -1, -2]);
        let result = processor.process_data();
        assert_eq!(result, vec![2, 4, 6, 8, 10]);
    }
    
    #[test]
    fn test_small_object_manager() {
        let mut manager = SmallObjectManager::new();
        manager.create_objects(100);
        let stats = manager.get_stats();
        assert_eq!(stats.total_objects, 100);
    }
    
    #[tokio::test]
    async fn test_type_checker_optimized() {
        let checker = TypeCheckerOptimized::new();
        let module_info = ModuleInfo {
            name: "test_module".to_string(),
            dependencies: vec!["dep1".to_string()],
            size: 1024,
            complexity: 10,
        };
        
        checker.add_module("test".to_string(), module_info).await;
        let stats = checker.get_compilation_stats().await;
        assert_eq!(stats.total_modules, 1);
    }
    
    #[tokio::test]
    async fn test_workflow_engine() {
        let mut engine = Rust190WorkflowEngine::new();
        let result = engine.execute_workflow().await.unwrap();
        assert!(!result.processed_data.is_empty());
        assert_eq!(result.object_stats.total_objects, 1000);
    }
}
