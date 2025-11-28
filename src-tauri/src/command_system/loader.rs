use super::builtin::{builtin_manifest, is_enabled};
use super::registry::COMMAND_REGISTRY;

pub async fn register_builtin_commands() -> usize {
    let mut registered = 0;

    for descriptor in builtin_manifest() {
        if is_enabled(descriptor.id) {
            COMMAND_REGISTRY.register((descriptor.builder)()).await;
            println!(
                "➡️ 已注册命令 {} ({})",
                descriptor.id, descriptor.description
            );
            registered += 1;
        } else {
            println!(
                "⚠️ 跳过命令 {}({})，已通过环境变量禁用",
                descriptor.id,
                descriptor.description
            );
        }
    }

    registered
}
