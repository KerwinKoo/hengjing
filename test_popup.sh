#!/bin/bash

# 恒境弹窗测试脚本
# 使用 target/release 中的 CLI 工具测试弹窗功能

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目根目录
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 默认构建类型和CLI类型
BUILD_TYPE="release"
CLI_TYPE="local"  # local 或 global
CLI_PATH="$PROJECT_ROOT/target/$BUILD_TYPE"

echo -e "${BLUE}🎯 恒境弹窗测试脚本${NC}"
echo -e "${BLUE}================================${NC}"

# 选择CLI类型
select_cli_type() {
    echo -e "${YELLOW}🔧 选择CLI类型:${NC}"
    echo -e "  ${GREEN}1.${NC} 本地编译版本 (从项目target目录)"
    echo -e "  ${GREEN}2.${NC} 全局安装版本 (系统PATH中)"
    echo ""

    while true; do
        read -p "请选择CLI类型 (1-2): " cli_choice
        case $cli_choice in
            1)
                CLI_TYPE="local"
                echo -e "${GREEN}✅ 已选择本地编译版本${NC}"
                select_build_type
                break
                ;;
            2)
                CLI_TYPE="global"
                echo -e "${GREEN}✅ 已选择全局安装版本${NC}"
                check_global_cli
                break
                ;;
            *)
                echo -e "${RED}❌ 无效选项，请选择 1 或 2${NC}"
                ;;
        esac
    done
    echo ""
}

# 选择构建类型（仅在使用本地CLI时）
select_build_type() {
    if [[ "$CLI_TYPE" != "local" ]]; then
        return
    fi

    echo -e "${YELLOW}🔧 选择构建类型:${NC}"
    echo -e "  ${GREEN}1.${NC} Release (推荐，性能最佳)"
    echo -e "  ${GREEN}2.${NC} Debug (包含调试信息)"
    echo ""

    while true; do
        read -p "请选择构建类型 (1-2): " build_choice
        case $build_choice in
            1)
                BUILD_TYPE="release"
                CLI_PATH="$PROJECT_ROOT/target/$BUILD_TYPE"
                echo -e "${GREEN}✅ 已选择 Release 构建${NC}"
                break
                ;;
            2)
                BUILD_TYPE="debug"
                CLI_PATH="$PROJECT_ROOT/target/$BUILD_TYPE"
                echo -e "${GREEN}✅ 已选择 Debug 构建${NC}"
                break
                ;;
            *)
                echo -e "${RED}❌ 无效选项，请选择 1 或 2${NC}"
                ;;
        esac
    done
    echo ""
}

# 检查全局CLI
check_global_cli() {
    echo -e "${YELLOW}🔍 检查全局CLI工具...${NC}"

    local continuum_found=false
    local dengxiaxia_found=false

    # 检查恒境
    if command -v 恒境 &> /dev/null; then
        echo -e "${GREEN}✅ 找到全局 恒境 CLI: $(which 恒境)${NC}"
        continuum_found=true
    else
        echo -e "${RED}❌ 未找到全局 恒境 CLI${NC}"
    fi

    # 检查等
    if command -v 等 &> /dev/null; then
        echo -e "${GREEN}✅ 找到全局 等 CLI: $(which 等)${NC}"
        dengxiaxia_found=true
    else
        echo -e "${RED}❌ 未找到全局 等 CLI${NC}"
    fi

    if [[ "$continuum_found" == false || "$dengxiaxia_found" == false ]]; then
        echo -e "${YELLOW}💡 全局CLI工具未完全安装，安装方法:${NC}"
        echo -e "${BLUE}   cargo install --path . --bins${NC}"
        echo -e "${YELLOW}   或者选择使用本地编译版本${NC}"
        echo ""

        echo -e "${BLUE}🔄 是否切换到本地编译版本？ (y/n)${NC}"
        read -p "请选择: " switch_choice
        if [[ "$switch_choice" =~ ^[Yy]$ ]]; then
            CLI_TYPE="local"
            select_build_type
            return
        else
            echo -e "${RED}❌ 无法继续，请先安装全局CLI工具${NC}"
            exit 1
        fi
    fi

    # 设置全局CLI路径
    CLI_PATH=""  # 全局CLI不需要路径前缀
    echo -e "${GREEN}✅ 全局CLI工具检查完成${NC}"
    echo ""
}

# 编译项目
compile_project() {
    if [[ "$CLI_TYPE" == "global" ]]; then
        echo -e "${YELLOW}⚠️  使用全局CLI，跳过编译步骤${NC}"
        return
    fi

    echo -e "${YELLOW}🔨 开始编译项目...${NC}"

    # 检查Cargo.toml是否存在
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        echo -e "${RED}❌ 未找到 Cargo.toml 文件${NC}"
        echo -e "${YELLOW}💡 请确保在Rust项目根目录中运行此脚本${NC}"
        exit 1
    fi

    # 检查cargo是否存在
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ 未找到 cargo 命令${NC}"
        echo -e "${YELLOW}💡 请先安装 Rust: https://rustup.rs/${NC}"
        exit 1
    fi

    # 根据构建类型编译
    if [[ "$BUILD_TYPE" == "release" ]]; then
        echo -e "${BLUE}📦 编译 Release 版本...${NC}"
        if cargo build --release; then
            echo -e "${GREEN}✅ Release 编译完成${NC}"
        else
            echo -e "${RED}❌ Release 编译失败${NC}"
            exit 1
        fi
    else
        echo -e "${BLUE}📦 编译 Debug 版本...${NC}"
        if cargo build; then
            echo -e "${GREEN}✅ Debug 编译完成${NC}"
        else
            echo -e "${RED}❌ Debug 编译失败${NC}"
            exit 1
        fi
    fi
    echo ""
}

# 检查CLI工具是否存在
check_cli_tools() {
    if [[ "$CLI_TYPE" == "global" ]]; then
        echo -e "${YELLOW}📋 检查全局CLI工具...${NC}"
        check_global_cli
        return
    fi

    echo -e "${YELLOW}📋 检查本地CLI工具 (${BUILD_TYPE})...${NC}"

    if [[ ! -f "$CLI_PATH/恒境" ]]; then
        echo -e "${RED}❌ 未找到 恒境 CLI工具${NC}"
        if [[ "$BUILD_TYPE" == "release" ]]; then
            echo -e "${YELLOW}💡 请先编译项目: cargo build --release${NC}"
        else
            echo -e "${YELLOW}💡 请先编译项目: cargo build${NC}"
        fi

        # 提供自动编译选项
        echo -e "${BLUE}🔨 是否现在编译项目？ (y/n)${NC}"
        read -p "请选择: " compile_choice
        if [[ "$compile_choice" =~ ^[Yy]$ ]]; then
            compile_project
        else
            exit 1
        fi
    fi

    if [[ ! -f "$CLI_PATH/等" ]]; then
        echo -e "${RED}❌ 未找到 等 CLI工具${NC}"
        if [[ "$BUILD_TYPE" == "release" ]]; then
            echo -e "${YELLOW}💡 请先编译项目: cargo build --release${NC}"
        else
            echo -e "${YELLOW}💡 请先编译项目: cargo build${NC}"
        fi

        # 提供自动编译选项
        echo -e "${BLUE}🔨 是否现在编译项目？ (y/n)${NC}"
        read -p "请选择: " compile_choice
        if [[ "$compile_choice" =~ ^[Yy]$ ]]; then
            compile_project
        else
            exit 1
        fi
    fi

    # 检查执行权限
    if [[ ! -x "$CLI_PATH/恒境" ]]; then
        echo -e "${YELLOW}⚠️  恒境 CLI工具没有执行权限，正在添加...${NC}"
        chmod +x "$CLI_PATH/恒境"
    fi

    if [[ ! -x "$CLI_PATH/等" ]]; then
        echo -e "${YELLOW}⚠️  等 CLI工具没有执行权限，正在添加...${NC}"
        chmod +x "$CLI_PATH/等"
    fi

    echo -e "${GREEN}✅ 本地CLI工具检查完成 (${BUILD_TYPE})${NC}"
    echo -e "   构建类型: ${BUILD_TYPE}"
    echo -e "   恒境: $CLI_PATH/恒境"
    echo -e "   等: $CLI_PATH/等"
}

# 检查测试JSON文件
check_test_files() {
    echo -e "${YELLOW}📋 检查测试文件...${NC}"
    
    if [[ ! -f "$PROJECT_ROOT/test_simple_popup.json" ]]; then
        echo -e "${RED}❌ 未找到 test_simple_popup.json${NC}"
        exit 1
    fi
    
    if [[ ! -f "$PROJECT_ROOT/test_markdown_popup.json" ]]; then
        echo -e "${RED}❌ 未找到 test_markdown_popup.json${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ 测试文件检查完成${NC}"
}

# 显示测试选项
show_test_options() {
    echo -e "${BLUE}🎨 可用的测试选项:${NC}"
    if [[ "$CLI_TYPE" == "global" ]]; then
        echo -e "${YELLOW}当前CLI类型: 全局安装版本${NC}"
    else
        echo -e "${YELLOW}当前CLI类型: 本地编译版本 (${BUILD_TYPE})${NC}"
    fi
    echo ""
    echo -e "  ${GREEN}1.${NC} 测试简单弹窗 (test_simple_popup.json)"
    echo -e "  ${GREEN}2.${NC} 测试Markdown弹窗 (test_markdown_popup.json)"
    echo -e "  ${GREEN}3.${NC} 测试自定义弹窗"
    echo -e "  ${GREEN}4.${NC} 启动前端测试环境"
    echo -e "  ${GREEN}5.${NC} 查看CLI工具帮助"
    echo -e "  ${GREEN}6.${NC} 切换CLI类型"
    echo -e "  ${GREEN}7.${NC} 安装/重新编译"
    echo -e "  ${GREEN}q.${NC} 退出"
    echo ""
}

# 显示JSON文件内容（兼容有无jq的情况）
show_json_content() {
    local file_path="$1"
    if command -v jq &> /dev/null; then
        cat "$file_path" | jq '.'
    else
        echo "JSON内容:"
        cat "$file_path"
    fi
}

# 获取CLI命令
get_cli_command() {
    local cli_name="$1"
    if [[ "$CLI_TYPE" == "global" ]]; then
        echo "$cli_name"
    else
        echo "$CLI_PATH/$cli_name"
    fi
}

# 测试简单弹窗
test_simple_popup() {
    echo -e "${YELLOW}🚀 启动简单弹窗测试...${NC}"
    echo -e "${BLUE}使用文件: test_simple_popup.json${NC}"

    # 显示文件内容
    echo -e "${YELLOW}📄 文件内容:${NC}"
    show_json_content "$PROJECT_ROOT/test_simple_popup.json"
    echo ""

    # 启动弹窗
    local cli_cmd=$(get_cli_command "等")
    echo -e "${GREEN}🎯 启动弹窗...${NC}"
    echo -e "${BLUE}执行命令: $cli_cmd --mcp-request test_simple_popup.json${NC}"
    if $cli_cmd --mcp-request "$PROJECT_ROOT/test_simple_popup.json"; then
        echo -e "${GREEN}✅ 弹窗测试完成${NC}"
    else
        echo -e "${RED}❌ 弹窗测试失败${NC}"
        echo -e "${YELLOW}💡 请检查CLI工具是否正常工作${NC}"
    fi
}

# 测试Markdown弹窗
test_markdown_popup() {
    echo -e "${YELLOW}🚀 启动Markdown弹窗测试...${NC}"
    echo -e "${BLUE}使用文件: test_markdown_popup.json${NC}"

    # 显示文件内容
    echo -e "${YELLOW}📄 文件内容:${NC}"
    show_json_content "$PROJECT_ROOT/test_markdown_popup.json"
    echo ""

    # 启动弹窗
    local cli_cmd=$(get_cli_command "等")
    echo -e "${GREEN}🎯 启动弹窗...${NC}"
    echo -e "${BLUE}执行命令: $cli_cmd --mcp-request test_markdown_popup.json${NC}"
    if $cli_cmd --mcp-request "$PROJECT_ROOT/test_markdown_popup.json"; then
        echo -e "${GREEN}✅ Markdown弹窗测试完成${NC}"
    else
        echo -e "${RED}❌ Markdown弹窗测试失败${NC}"
        echo -e "${YELLOW}💡 请检查CLI工具是否正常工作${NC}"
    fi
}

# 测试自定义弹窗
test_custom_popup() {
    echo -e "${YELLOW}🚀 创建自定义弹窗测试...${NC}"
    
    # 创建临时测试文件
    TEMP_FILE="/tmp/custom_popup_test.json"
    
    cat > "$TEMP_FILE" << 'EOF'
{
  "id": "custom-test-001",
  "message": "# 🎨 自定义弹窗测试\n\n这是一个自定义的弹窗测试，用于验证弹窗功能的完整性。\n\n## ✨ 测试功能\n- 头部固定显示\n- 工具栏固定显示\n- 图片组件渲染\n- 输入框组件\n- 禁止选中非内容区域\n- Markdown紧凑渲染\n\n## 🔧 操作说明\n1. 测试主题切换按钮\n2. 测试打开主界面按钮\n3. 测试预定义选项选择\n4. 测试文本输入功能\n5. 测试图片粘贴功能\n\n```typescript\n// 示例代码\ninterface PopupTest {\n  header: 'fixed'\n  toolbar: 'fixed'\n  content: 'scrollable'\n  images: 'component-rendered'\n  input: 'component-based'\n}\n```\n\n> **注意**: 请测试所有交互功能以确保弹窗工作正常。",
  "predefined_options": [
    "🎨 测试主题切换",
    "🏠 测试主界面按钮", 
    "📝 测试文本输入",
    "🖼️ 测试图片功能",
    "⚡ 测试快捷键",
    "✅ 测试完成",
    "❌ 发现问题"
  ],
  "is_markdown": true
}
EOF
    
    echo -e "${YELLOW}📄 自定义测试内容:${NC}"
    show_json_content "$TEMP_FILE"
    echo ""
    
    # 启动弹窗
    local cli_cmd=$(get_cli_command "等")
    echo -e "${GREEN}🎯 启动自定义弹窗...${NC}"
    echo -e "${BLUE}执行命令: $cli_cmd --mcp-request $TEMP_FILE${NC}"
    if $cli_cmd --mcp-request "$TEMP_FILE"; then
        echo -e "${GREEN}✅ 自定义弹窗测试完成${NC}"
    else
        echo -e "${RED}❌ 自定义弹窗测试失败${NC}"
        echo -e "${YELLOW}💡 请检查CLI工具是否正常工作${NC}"
    fi

    # 清理临时文件
    rm -f "$TEMP_FILE"
}

# 启动前端测试环境
start_frontend_test() {
    echo -e "${YELLOW}🚀 启动前端测试环境...${NC}"
    echo -e "${BLUE}测试环境将在 http://localhost:5174 启动${NC}"
    echo -e "${YELLOW}💡 按 Ctrl+C 停止测试环境${NC}"
    echo ""

    # 检查pnpm是否存在
    if ! command -v pnpm &> /dev/null; then
        echo -e "${RED}❌ 未找到 pnpm 命令${NC}"
        echo -e "${YELLOW}💡 请先安装 pnpm: npm install -g pnpm${NC}"
        return 1
    fi

    # 检查package.json是否存在
    if [[ ! -f "$PROJECT_ROOT/package.json" ]]; then
        echo -e "${RED}❌ 未找到 package.json 文件${NC}"
        return 1
    fi

    # 启动测试环境
    cd "$PROJECT_ROOT" && pnpm test:ui
}

# 显示CLI帮助
show_cli_help() {
    echo -e "${YELLOW}📖 CLI工具帮助信息:${NC}"
    echo ""

    local continuum_cmd=$(get_cli_command "恒境")
    local dengxiaxia_cmd=$(get_cli_command "等")

    echo -e "${BLUE}恒境 CLI:${NC}"
    echo -e "${BLUE}命令: $continuum_cmd${NC}"
    if $continuum_cmd --help 2>/dev/null; then
        echo -e "${GREEN}✅ 帮助信息显示完成${NC}"
    else
        echo -e "${YELLOW}⚠️  恒境 CLI 无帮助信息或不支持 --help 参数${NC}"
        echo -e "${BLUE}尝试直接运行:${NC} $continuum_cmd"
    fi
    echo ""

    echo -e "${BLUE}等 CLI:${NC}"
    echo -e "${BLUE}命令: $dengxiaxia_cmd${NC}"
    if $dengxiaxia_cmd --help 2>/dev/null; then
        echo -e "${GREEN}✅ 帮助信息显示完成${NC}"
    else
        echo -e "${YELLOW}⚠️  等 CLI 无帮助信息或不支持 --help 参数${NC}"
        echo -e "${BLUE}尝试直接运行:${NC} $dengxiaxia_cmd"
        echo -e "${BLUE}MCP请求参数:${NC} $dengxiaxia_cmd --mcp-request <json_file>"
    fi
}

# 切换CLI类型
switch_cli_type() {
    echo -e "${YELLOW}🔄 切换CLI类型${NC}"
    if [[ "$CLI_TYPE" == "global" ]]; then
        echo -e "当前CLI类型: 全局安装版本"
    else
        echo -e "当前CLI类型: 本地编译版本 (${BUILD_TYPE})"
    fi
    echo ""

    if [[ "$CLI_TYPE" == "global" ]]; then
        CLI_TYPE="local"
        echo -e "${GREEN}✅ 已切换到本地编译版本${NC}"
        select_build_type
    else
        CLI_TYPE="global"
        echo -e "${GREEN}✅ 已切换到全局安装版本${NC}"
        check_global_cli
    fi
    echo ""
}

# 安装/重新编译
install_or_compile() {
    if [[ "$CLI_TYPE" == "global" ]]; then
        echo -e "${YELLOW}🔨 安装全局CLI工具...${NC}"
        echo -e "${BLUE}执行命令: cargo install --path . --bins${NC}"

        if cargo install --path . --bins; then
            echo -e "${GREEN}✅ 全局CLI工具安装完成${NC}"
            check_global_cli
        else
            echo -e "${RED}❌ 全局CLI工具安装失败${NC}"
        fi
    else
        echo -e "${YELLOW}🔨 重新编译本地项目 (${BUILD_TYPE})...${NC}"
        compile_project
        check_cli_tools
    fi
}

# 主函数
main() {
    # 选择CLI类型
    select_cli_type

    # 检查依赖
    check_cli_tools
    check_test_files

    echo ""

    # 主循环
    while true; do
        show_test_options
        read -p "请选择测试选项 (1-7, q): " choice
        echo ""

        case $choice in
            1)
                test_simple_popup
                ;;
            2)
                test_markdown_popup
                ;;
            3)
                test_custom_popup
                ;;
            4)
                start_frontend_test
                ;;
            5)
                show_cli_help
                ;;
            6)
                switch_cli_type
                ;;
            7)
                install_or_compile
                ;;
            q|Q)
                echo -e "${GREEN}👋 测试结束，再见！${NC}"
                exit 0
                ;;
            *)
                echo -e "${RED}❌ 无效选项，请重新选择${NC}"
                ;;
        esac

        echo ""
        echo -e "${YELLOW}按回车键继续...${NC}"
        read
        echo ""
    done
}

# 检查依赖工具
echo -e "${BLUE}🔍 检查依赖工具...${NC}"
if ! command -v jq &> /dev/null; then
    echo -e "${YELLOW}⚠️  建议安装 jq 以获得更好的JSON显示效果${NC}"
    echo -e "${YELLOW}   macOS: brew install jq${NC}"
    echo -e "${YELLOW}   Ubuntu/Debian: sudo apt install jq${NC}"
    echo -e "${YELLOW}   CentOS/RHEL: sudo yum install jq${NC}"
    echo ""
else
    echo -e "${GREEN}✅ jq 已安装${NC}"
fi

if ! command -v pnpm &> /dev/null; then
    echo -e "${YELLOW}⚠️  建议安装 pnpm 以使用前端测试环境${NC}"
    echo -e "${YELLOW}   安装命令: npm install -g pnpm${NC}"
    echo ""
else
    echo -e "${GREEN}✅ pnpm 已安装${NC}"
fi
echo ""

# 运行主函数
main "$@"
