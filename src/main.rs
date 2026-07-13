use anyhow::Result;
use clap::{Parser, Subcommand};

// 引入模块
mod commands;
mod objects;
mod storage;

#[derive(Parser)]
#[command(name = "minigit", about = "A pure local mini version of Git in Rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化 minigit 仓库
    Init,
    /// 计算文件哈希并写入对象库
    HashObject {
        #[arg(short, long)]
        write: bool,
        file: String,
    },
    /// 将文件添加到暂存区
    Add { file: String },
    /// 提交暂存区到仓库
    Commit {
        #[arg(short, long)]
        message: String,
    },
    /// 查看提交历史
    Log,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init => commands::init()?,
        Commands::HashObject { write, file } => commands::hash_object(&file, write)?,
        Commands::Add { file } => commands::add(&file)?,
        Commands::Commit { message } => commands::commit(&message)?,
        Commands::Log => commands::log()?,
    }
    
    Ok(())
}