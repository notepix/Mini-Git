#!/bin/bash

# 先统一编译一次！
echo "=== 编译项目中... ==="
cargo build

echo -e "\n=== 1. 初始化空仓库 ==="
# 直接运行编译好的二进制文件！
./target/debug/minigit init

echo -e "\n=== 2. 创建测试文件 V1 ==="
echo "Hello Minigit!" > fileA.txt
echo "Rust Course" > fileB.txt
cat fileA.txt
cat fileB.txt

echo -e "\n=== 3. 将文件添加到暂存区 ==="
./target/debug/minigit add fileA.txt
./target/debug/minigit add fileB.txt

echo -e "\n=== 4. 验证底层对象生成 ==="
tree .minigit/objects  # 记得先用 sudo apt install tree 安装一下

echo -e "\n=== 5. 执行第一次提交 ==="
./target/debug/minigit commit -m "Initial commit: Add fileA and fileB"

echo -e "\n=== 6. 修改文件并执行第二次提交 ==="
echo "Modified content" >> fileA.txt
./target/debug/minigit add fileA.txt
./target/debug/minigit commit -m "Update fileA content"

echo -e "\n=== 7. 打印提交流程验证连通性 ==="
./target/debug/minigit log