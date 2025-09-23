//! # 中间件示例 / Middleware Examples
//!
//! 本模块展示了工作流中间件的使用方法
//! This module demonstrates how to use workflow middleware

//use std::time::Duration;

/// 运行中间件示例 / Run middleware examples
pub async fn run_middleware_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 中间件示例 / Middleware Examples");
    println!("==================================");

    // 模拟中间件功能 / Simulate middleware functionality
    println!("\n1. 认证中间件示例 / Authentication Middleware Example");
    println!("   - 模拟 API 密钥验证 / Simulating API key validation");
    println!("   - 模拟用户权限检查 / Simulating user permission check");

    println!("\n2. 日志中间件示例 / Logging Middleware Example");
    println!("   - 记录请求信息 / Logging request information");
    println!("   - 记录响应状态 / Logging response status");

    println!("\n3. 监控中间件示例 / Monitoring Middleware Example");
    println!("   - 收集性能指标 / Collecting performance metrics");
    println!("   - 生成监控报告 / Generating monitoring reports");

    println!("\n4. 限流中间件示例 / Rate Limiting Middleware Example");
    println!("   - 限制请求频率 / Limiting request frequency");
    println!("   - 处理限流异常 / Handling rate limit exceptions");

    println!("\n✅ 中间件示例运行完成 / Middleware examples completed successfully");
    Ok(())
}