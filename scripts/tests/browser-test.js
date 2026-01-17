/**
 * Tauri 自动化测试脚本 - 浏览器控制台版本
 *
 * 在 Tauri 窗口的开发者控制台中运行此脚本
 *
 * 使用方法:
 * 1. 在 Tauri 窗口中按 F12 打开开发者控制台
 * 2. 复制并粘贴以下代码
 * 3. 按回车执行
 */

(async function runTauriTests() {
  const colors = {
    reset: '%c',
    bright: 'font-weight: bold',
    red: 'color: #ef4444',
    green: 'color: #10b981',
    yellow: 'color: #f59e0b',
    blue: 'color: #3b82f6',
    cyan: 'color: #06b6d4',
  };

  const log = {
    header: (msg) => console.log(`%c${msg}`, 'font-weight: bold; color: #06b6d4'),
    section: (msg) => console.log(`%c${msg}`, 'font-weight: bold'),
    success: (msg) => console.log(`%c✓ ${msg}`, 'color: #10b981'),
    error: (msg) => console.log(`%c✗ ${msg}`, 'color: #ef4444'),
    warn: (msg) => console.log(`%c○ ${msg}`, 'color: #f59e0b'),
    info: (msg) => console.log(`%c▶ ${msg}`, 'color: #3b82f6'),
    result: (msg) => console.log(`  ${msg}`),
  };

  const results = {
    passed: [],
    failed: [],
    skipped: [],
  };

  function recordPass(name, detail = '') {
    results.passed.push({ name, detail });
    log.success(`${name}${detail ? ` - ${detail}` : ''}`);
  }

  function recordFail(name, error) {
    results.failed.push({ name, error });
    log.error(`${name}`);
    log.result(`错误: ${error}`);
  }

  function recordSkip(name, reason) {
    results.skipped.push({ name, reason });
    log.warn(`${name} - ${reason}`);
  }

  // 检查 Tauri 是否可用
  function isTauriAvailable() {
    try {
      return typeof window.__TAURI__ !== 'undefined' ||
             typeof window.__TAURI_INTERNALS__ !== 'undefined';
    } catch {
      return false;
    }
  }

  // 获取 Tauri invoke 函数
  function getInvoke() {
    if (typeof window.__TAURI__?.core?.invoke === 'function') {
      return window.__TAURI__.core.invoke;
    }
    if (typeof window.__TAURI_INTERNALS__?.core?.invoke === 'function') {
      return window.__TAURI_INTERNALS__.core.invoke;
    }
    return null;
  }

  log.header('='.repeat(60));
  log.header('TAURI 自动化测试');
  log.header('='.repeat(60));
  console.log();

  // 测试 1: Tauri 可用性检测
  log.section('\n--- 测试 1: Tauri 可用性检测 ---');
  if (isTauriAvailable()) {
    recordPass('Tauri 可用性检测', 'Tauri 环境正常');
  } else {
    recordFail('Tauri 可用性检测', 'Tauri 不可用（非桌面环境）');
    log.error('请在 Tauri 桌面应用中运行此测试！');
    return;
  }

  const invoke = getInvoke();
  if (!invoke) {
    recordFail('Tauri invoke 函数', '无法获取 invoke 函数');
    return;
  }
  recordPass('Tauri invoke 函数', 'invoke 函数可用');

  // 测试辅助函数
  async function invokeCommand(cmd, args = {}) {
    try {
      const result = await invoke(cmd, args);
      return { success: true, data: result };
    } catch (error) {
      return { success: false, error: error.message || error.toString() };
    }
  }

  // 测试 2: 获取应用配置
  log.section('\n--- 测试 2: 获取应用配置 ---');
  const configResult = await invokeCommand('get_app_config');
  if (configResult.success) {
    log.result(`配置: ${JSON.stringify(configResult.data, null, 2)}`);
    recordPass('获取应用配置');
  } else {
    recordFail('获取应用配置', configResult.error);
  }

  // 测试 3: 检查后端安装状态
  log.section('\n--- 测试 3: 检查后端安装状态 ---');
  const installResult = await invokeCommand('check_backend_installation');
  if (installResult.success) {
    log.result(`安装状态: ${JSON.stringify(installResult.data, null, 2)}`);
    recordPass('检查后端安装状态');
  } else {
    recordFail('检查后端安装状态', installResult.error);
  }

  // 测试 4: 检查后端运行状态
  log.section('\n--- 测试 4: 检查后端运行状态 ---');
  const statusResult = await invokeCommand('check_backend_status');
  if (statusResult.success) {
    log.result(`运行状态: ${JSON.stringify(statusResult.data, null, 2)}`);
    recordPass('检查后端运行状态');
  } else {
    recordFail('检查后端运行状态', statusResult.error);
  }

  // 测试 5: 获取下载目录
  log.section('\n--- 测试 5: 获取下载目录 ---');
  const dirResult = await invokeCommand('get_download_dir_path');
  if (dirResult.success) {
    log.result(`下载目录: ${dirResult.data}`);
    recordPass('获取下载目录');
  } else {
    recordFail('获取下载目录', dirResult.error);
  }

  // 测试 6: 启动后端（如果未运行）
  log.section('\n--- 测试 6: 启动后端测试 ---');
  if (statusResult.success && statusResult.data.is_running) {
    log.result('后端已在运行，跳过启动测试');
    recordSkip('启动后端', '后端已在运行');
  } else {
    log.info('正在启动后端...');
    const startResult = await invokeCommand('start_backend');
    if (startResult.success) {
      log.result(`启动结果: ${startResult.data}`);

      // 等待后端启动
      log.info('等待 3 秒让后端启动...');
      await new Promise(r => setTimeout(r, 3000));

      // 检查启动后的状态
      const newStatusResult = await invokeCommand('check_backend_status');
      if (newStatusResult.success && newStatusResult.data.is_running) {
        log.result(`后端已启动，端口: ${newStatusResult.data.port}`);
        recordPass('启动后端', `端口: ${newStatusResult.data.port}`);
      } else {
        recordFail('启动后端', '启动后状态检查失败');
      }
    } else {
      recordFail('启动后端', startResult.error);
    }
  }

  // 测试 7: 获取后端日志
  log.section('\n--- 测试 7: 获取后端日志 ---');
  const logsResult = await invokeCommand('get_backend_logs');
  if (logsResult.success) {
    const logs = logsResult.data;
    if (logs && logs.length > 0) {
      log.result(`最近的日志 (${logs.length} 条):`);
      logs.slice(-5).forEach(log => log.result(`  ${log}`));
    } else {
      log.result('暂无日志');
    }
    recordPass('获取后端日志');
  } else {
    recordFail('获取后端日志', logsResult.error);
  }

  // 打印总结
  log.header('\n' + '='.repeat(60));
  log.header('测试总结');
  log.header('='.repeat(60));

  console.log(`\n  %c通过: ${results.passed.length}`, 'color: #10b981; font-weight: bold');
  console.log(`  %c失败: ${results.failed.length}`, 'color: #ef4444; font-weight: bold');
  console.log(`  %c跳过: ${results.skipped.length}`, 'color: #f59e0b; font-weight: bold');
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

  log.info('\n测试完成！');

  return results;
})();
