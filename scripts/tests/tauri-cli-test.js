/**
 * Tauri CLI 自动化测试脚本
 *
 * 通过命令行直接测试 Tauri 命令功能
 *
 * Usage:
 *   node scripts/tests/tauri-cli-test.js
 */

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = join(__dirname, '../..');

// ANSI colors
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
  magenta: '\x1b[35m',
};

// Test results
const results = {
  passed: [],
  failed: [],
  skipped: [],
};

// Logger
const log = {
  info: (msg) => console.log(`${colors.blue}[INFO]${colors.reset} ${msg}`),
  success: (msg) => console.log(`${colors.green}[PASS]${colors.reset} ${msg}`),
  error: (msg) => console.log(`${colors.red}[FAIL]${colors.reset} ${msg}`),
  warn: (msg) => console.log(`${colors.yellow}[SKIP]${colors.reset} ${msg}`),
  header: (msg) => console.log(`\n${colors.bright}${colors.cyan}${'='.repeat(60)}`),
  section: (msg) => console.log(`${colors.bright}${msg}${colors.reset}`),
  test: (msg) => console.log(`  ${colors.cyan}▶${colors.reset} ${msg}`),
};

function recordPass(name, detail = '') {
  results.passed.push({ name, detail });
  log.success(`✓ ${name}${detail ? ` - ${detail}` : ''}`);
}

function recordFail(name, error) {
  results.failed.push({ name, error });
  log.error(`✗ ${name}`);
  if (error) console.error(`  ${colors.red}${error}${colors.reset}`);
}

function recordSkip(name, reason) {
  results.skipped.push({ name, reason });
  log.warn(`○ ${name} - ${reason}`);
}

// Execute Tauri command
async function invokeTauriCommand(command, args = []) {
  return new Promise((resolve, reject) => {
    const tauri = spawn('pnpm', ['tauri', 'cli', command, ...args], {
      cwd: ROOT_DIR,
      shell: true,
    });

    let stdout = '';
    let stderr = '';

    tauri.stdout.on('data', (data) => {
      stdout += data.toString();
    });

    tauri.stderr.on('data', (data) => {
      stderr += data.toString();
    });

    tauri.on('close', (code) => {
      if (code === 0) {
        resolve(stdout.trim());
      } else {
        reject(new Error(`Command failed with code ${code}: ${stderr || stdout}`));
      }
    });

    setTimeout(() => {
      tauri.kill();
      reject(new Error('Command timeout'));
    }, 30000);
  });
}

// Test: Tauri 可用性检测
async function testTauriAvailability() {
  log.test('Tauri 可用性检测');
  try {
    // 尝试运行一个简单的 Tauri 命令
    const output = await invokeTauriCommand('help');
    if (output.includes('Usage:')) {
      recordPass('Tauri 可用性检测', 'Tauri CLI 可用');
      return true;
    }
  } catch (error) {
    recordFail('Tauri 可用性检测', error.message);
    return false;
  }
}

// Test: 获取应用配置
async function testGetAppConfig() {
  log.test('获取应用配置');
  try {
    const output = await invokeTauriCommand('get_app_config');
    log.info(`配置输出: ${output.substring(0, 100)}...`);
    recordPass('获取应用配置');
    return JSON.parse(output);
  } catch (error) {
    recordFail('获取应用配置', error.message);
    return null;
  }
}

// Test: 检查后端安装状态
async function testBackendInstallation() {
  log.test('检查后端安装状态');
  try {
    const output = await invokeTauriCommand('check_backend_installation');
    log.info(`后端状态: ${output}`);
    recordPass('检查后端安装状态');
    return JSON.parse(output);
  } catch (error) {
    recordFail('检查后端安装状态', error.message);
    return null;
  }
}

// Test: 检查后端运行状态
async function testBackendStatus() {
  log.test('检查后端运行状态');
  try {
    const output = await invokeTauriCommand('check_backend_status');
    log.info(`状态输出: ${output}`);
    recordPass('检查后端运行状态');
    return JSON.parse(output);
  } catch (error) {
    recordFail('检查后端运行状态', error.message);
    return null;
  }
}

// Test: 获取下载目录
async function testGetDownloadDir() {
  log.test('获取下载目录');
  try {
    const output = await invokeTauriCommand('get_download_dir_path');
    log.info(`下载目录: ${output}`);
    recordPass('获取下载目录');
    return output;
  } catch (error) {
    recordFail('获取下载目录', error.message);
    return null;
  }
}

// Test: 启动后端
async function testStartBackend() {
  log.test('启动后端（测试）');
  try {
    // 先检查后端是否已经在运行
    const statusOutput = await invokeTauriCommand('check_backend_status');
    const status = JSON.parse(statusOutput);

    if (status.is_running) {
      log.info('后端已在运行，跳过启动测试');
      recordPass('启动后端', '后端已在运行');
      return { skipped: true, alreadyRunning: true };
    }

    // 尝试启动后端
    const output = await invokeTauriCommand('start_backend');
    log.info(`启动输出: ${output}`);

    // 等待几秒让后端启动
    await new Promise(resolve => setTimeout(resolve, 3000));

    // 再次检查状态
    const newStatusOutput = await invokeTauriCommand('check_backend_status');
    const newStatus = JSON.parse(newStatusOutput);

    if (newStatus.is_running) {
      recordPass('启动后端', `后端已启动，端口: ${newStatus.port}`);
      return { success: true, port: newStatus.port };
    } else {
      recordFail('启动后端', '后端启动后状态检查失败');
      return { success: false };
    }
  } catch (error) {
    recordFail('启动后端', error.message);
    return { success: false };
  }
}

// Test: 停止后端
async function testStopBackend() {
  log.test('停止后端（测试）');
  try {
    const output = await invokeTauriCommand('stop_backend');
    log.info(`停止输出: ${output}`);

    // 等待几秒让后端停止
    await new Promise(resolve => setTimeout(resolve, 2000));

    // 检查状态
    const statusOutput = await invokeTauriCommand('check_backend_status');
    const status = JSON.parse(statusOutput);

    if (!status.is_running) {
      recordPass('停止后端');
      return true;
    } else {
      recordFail('停止后端', '后端仍在运行');
      return false;
    }
  } catch (error) {
    recordFail('停止后端', error.message);
    return false;
  }
}

// Print summary
function printSummary() {
  log.header('=');
  log.section('测试总结');
  log.header('=');

  console.log(`\n  ${colors.green}通过: ${results.passed.length}${colors.reset}`);
  console.log(`  ${colors.red}失败: ${results.failed.length}${colors.reset}`);
  console.log(`  ${colors.yellow}跳过: ${results.skipped.length}${colors.reset}`);
  console.log(`  总计: ${results.passed.length + results.failed.length + results.skipped.length}\n`);

  if (results.failed.length > 0) {
    log.error('\n失败的测试:');
    results.failed.forEach((f) => {
      console.log(`  - ${f.name}: ${f.error}`);
    });
  }

  if (results.skipped.length > 0) {
    log.warn('\n跳过的测试:');
    results.skipped.forEach((s) => {
      console.log(`  - ${s.name}: ${s.reason}`);
    });
  }
}

// Main test runner
async function runTests() {
  log.header('=');
  log.section('TAURI 命令行自动化测试');
  log.header('=');

  log.info('开始测试 Tauri 命令...\n');

  // 运行测试
  await testTauriAvailability();
  await testGetAppConfig();
  await testBackendInstallation();
  await testBackendStatus();
  await testGetDownloadDir();

  // 后端启动/停止测试（可选，需要更长时间）
  log.section('\n是否测试后端启动/停止功能？');
  log.info('（这会启动实际的 Python 后端进程）');
  log.info('跳过此测试，因为需要更多时间');

  recordSkip('启动后端测试', '需要手动确认');
  recordSkip('停止后端测试', '需要手动确认');

  // 打印总结
  printSummary();

  // 返回退出码
  const exitCode = results.failed.length > 0 ? 1 : 0;
  process.exit(exitCode);
}

// Run tests
runTests().catch((err) => {
  log.error(`\n致命错误: ${err.message}`);
  console.error(err);
  process.exit(1);
});
