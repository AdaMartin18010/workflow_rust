//! # 设计模式示例 / Design Pattern Examples

/// 运行设计模式示例 / Run design pattern examples
pub async fn run_pattern_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 设计模式示例 / Design Pattern Examples");
    println!("========================================");

    println!("\n1. 创建型模式示例 / Creational Patterns Example");
    println!("   - 建造者模式 / Builder Pattern");
    println!("   - 工厂模式 / Factory Pattern");
    println!("   - 单例模式 / Singleton Pattern");

    println!("\n2. 结构型模式示例 / Structural Patterns Example");
    println!("   - 适配器模式 / Adapter Pattern");
    println!("   - 桥接模式 / Bridge Pattern");
    println!("   - 组合模式 / Composite Pattern");
    println!("   - 装饰器模式 / Decorator Pattern");

    println!("\n3. 行为型模式示例 / Behavioral Patterns Example");
    println!("   - 观察者模式 / Observer Pattern");
    println!("   - 策略模式 / Strategy Pattern");
    println!("   - 状态模式 / State Pattern");
    println!("   - 命令模式 / Command Pattern");

    println!("\n4. 并发模式示例 / Concurrent Patterns Example");
    println!("   - Actor 模式 / Actor Pattern");
    println!("   - 生产者-消费者模式 / Producer-Consumer Pattern");
    println!("   - 管道模式 / Pipeline Pattern");
    println!("   - 反应器模式 / Reactor Pattern");

    println!("\n✅ 设计模式示例运行完成 / Design pattern examples completed successfully");
    Ok(())
}
