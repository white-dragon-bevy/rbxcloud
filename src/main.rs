mod cli;

use clap::Parser;
use cli::Cli;
use std::process;

/// 加载项目目录中所有的 .env.roblox 和 .env.roblox.* 文件
fn load_dotenv_files() {
    use std::fs;
    use std::path::Path;

    // 获取当前目录
    let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());

    // 读取目录中的所有文件
    if let Ok(entries) = fs::read_dir(&current_dir) {
        let mut env_files: Vec<String> = entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name() {
                        let file_name_str = file_name.to_string_lossy().to_string();
                        // 匹配 .env.roblox 或 .env.roblox.* 文件
                        if file_name_str == ".env.roblox" || file_name_str.starts_with(".env.roblox.") {
                            return Some(file_name_str);
                        }
                    }
                }
                None
            })
            .collect();

        // 排序以确保加载顺序一致
        // .env.roblox 文件最后加载，这样它可以覆盖其他环境文件的值
        env_files.sort_by(|a, b| {
            if a == ".env.roblox" {
                std::cmp::Ordering::Greater
            } else if b == ".env.roblox" {
                std::cmp::Ordering::Less
            } else {
                a.cmp(b)
            }
        });

        // 加载所有找到的 .env.roblox 文件
        for env_file in env_files {
            let env_path = current_dir.join(&env_file);
            match dotenvy::from_path(&env_path) {
                Ok(_) => eprintln!("已加载环境变量文件: {}", env_file),
                Err(e) => eprintln!("加载环境变量文件 {} 失败: {}", env_file, e),
            }
        }
    }

    // 检查并提示代理配置
    check_proxy_config();
}

/// 检查代理配置并在需要时输出提示
fn check_proxy_config() {
    let proxy_vars = [
        "HTTP_PROXY", "http_proxy",
        "HTTPS_PROXY", "https_proxy",
        "ALL_PROXY", "all_proxy",
    ];

    let mut found_proxy = false;
    for var in &proxy_vars {
        if let Ok(value) = std::env::var(var) {
            if !value.is_empty() {
                eprintln!("检测到代理配置: {}={}", var, value);
                found_proxy = true;
            }
        }
    }

    if found_proxy {
        eprintln!("代理配置已启用，所有 HTTP/HTTPS 请求将通过代理服务器。");
    }
}

#[tokio::main]
async fn main() {
    // 加载所有 .env.roblox 文件
    load_dotenv_files();

    let cli_args = Cli::parse();

    match cli_args.run().await {
        Ok(str) => {
            if let Some(s) = str {
                println!("{s}");
            }
        }
        Err(err) => {
            eprintln!("{err:?}");
            process::exit(1);
        }
    }
}
