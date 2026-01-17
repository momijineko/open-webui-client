/**
 * Tauri v2 自动化测试脚本 - 浏览器控制台版本
 *
 * 在 Tauri 窗口的开发者控制台中运行此脚本
 *
 * 使用方法:
 * 1. 在 Tauri 窗口中按 F12 打开开发者控制台
 * 2. 复制并粘贴以下代码
 * 3. 按回车执行
 */

(async function runTauriTests() {
  const log = {
    header: (msg) => console.log(`%c${msg}`, 'font-weight: bold; color: #06b6d4'),
    success: (msg) => console.log(`%c✓ ${msg}`, 'color: #10b981'),
    error: (msg) => console.log(`%c✗ ${msg}`, 'color: #ef4444'),
    warn: (msg) => console.log(`%c○ ${msg}`, 'color: #f59e0b'),
    info: (msg) => console.log(`%c▶ ${msg}`, 'color: #3b82f6'),
    result: (msg) => console.log(`  ${msg}`),
  };

  const results = { passed: [], failed: [], skipped: [] };

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

  log.header('='.repeat(60));
  log.header('TAURI v2 自动化测试');
  log.header('='.repeat(60));

  // 检测 Tauri 环境
  log.info('\n检测 Tauri 环境...');
  const hasTauri = typeof window !== 'undefined' && (
    '__TAURI__' in window || '__TAURI_INTERNALS__' in window
  );

  if (!hasTauri) {
    recordFail('Tauri 环境', '未检测到 Tauri');
    log.error('请在 Tauri 桌面应用中运行此测试！');
    return;
  }
  recordPass('Tauri 环境', '已检测到');

  // 尝试多种方式获取 invoke 函数
  let invoke = null;

  // 方法 1: 通过 __TAURI_INTERNALS__ (Tauri v2)
  if (window.__TAURI_INTERNALS__) {
    log.result('找到 __TAURI_INTERNALS__');
    const internals = window.__TAURI_INTERNALS__;

    // 尝试不同的路径
    if (internals.core?.invoke) {
      invoke = internals.core.invoke;
      log.result('使用: __TAURI_INTERNALS__.core.invoke');
    } else if (internals.invoke) {
      invoke = internals.invoke;
      log.result('使用: __TAURI_INTERNALS__.invoke');
    } else if (internals.app?.invoke) {
      invoke = internals.app.invoke;
      log.result('使用: __TAURI_INTERNALS__.app.invoke');
    }
  }

  // 方法 2: 通过 __TAURI__ (Tauri v1/v2)
  if (!invoke && window.__TAURI__) {
    log.result('找到 __TAURI__');
    const tauri = window.__TAURI__;

    if (tauri.core?.invoke) {
      invoke = tauri.core.invoke;
      log.result('使用: __TAURI__.core.invoke');
    } else if (tauri.invoke) {
      invoke = tauri.invoke;
      log.result('使用: __TAURI__.invoke');
    } else if (tauri.tauri?.invoke) {
      invoke = tauri.tauri.invoke;
      log.result('使用: __TAURI__.tauri.invoke');
    }
  }

  // 方法 3: 检查全局 invoke
  if (!invoke && typeof window.invoke === 'function') {
    invoke = window.invoke;
    log.result('使用: window.invoke');
  }

  if (!invoke) {
    recordFail('invoke 函数', '无法获取 invoke 函数');
    log.error('\n可用的全局对象:');
    log.result('window.__TAURI__: ' + typeof window.__TAURI__);
    log.result('window.__TAURI_INTERNALS__: ' + typeof window.__TAURI_INTERNALS__);

    // 尝试打印对象结构
    if (window.__TAURI_INTERNALS__) {
      log.result('\n__TAURI_INTERNALS__ 结构:');
      console.dir(window.__TAURI_INTERNALS__);
    }
    if (window.__TAURI__) {
      log.result('\n__TAURI__ 结构:');
      console.dir(window.__TAURI__);
    }
    return;
  }

  recordPass('invoke 函数', '已获取');

  // 测试调用函数
  async function test(cmd, args = {}) {
    try {
      const data = await invoke(cmd, args);
      return { success: true, data };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  }

  // 测试 1: 获取应用配置
  log.info('\n测试 1: 获取应用配置...');
  const cfg = await test('get_app_config');
  if (cfg.success) {
    log.result(`配置: ${JSON.stringify(cfg.data, null, 2)}`);
    recordPass('获取应用配置');
  } else {
    recordFail('获取应用配置', cfg.error);
  }

  // 测试 2: 检查后端安装
  log.info('\n测试 2: 检查后端安装...');
  const inst = await test('check_backend_installation');
  if (inst.success) {
    log.result(`安装状态: ${JSON.stringify(inst.data, null, 2)}`);
    recordPass('检查后端安装');
  } else {
    recordFail('检查后端安装', inst.error);
  }

  // 测试 3: 检查后端状态
  log.info('\n测试 3: 检查后端状态...');
  const st = await test('check_backend_status');
  if (st.success) {
    log.result(`运行状态: ${JSON.stringify(st.data, null, 2)}`);
    recordPass('检查后端状态');
  } else {
    recordFail('检查后端状态', st.error);
  }

  // 测试 4: 获取下载目录
  log.info('\n测试 4: 获取下载目录...');
  const dir = await test('get_download_dir_path');
  if (dir.success) {
    log.result(`下载目录: ${dir.data}`);
    recordPass('获取下载目录');
  } else {
    recordFail('获取下载目录', dir.error);
  }

  // 测试 5: 启动后端
  log.info('\n测试 5: 启动后端...');
  if (st.success && st.data.is_running) {
    log.result('后端已在运行，跳过启动测试');
    recordSkip('启动后端', '后端已在运行');
  } else {
    const start = await test('start_backend');
    if (start.success) {
      log.result(`启动结果: ${start.data}`);
      log.info('等待 3 秒让后端启动...');
      await new Promise(r => setTimeout(r, 3000));
      const st2 = await test('check_backend_status');
      if (st2.success && st2.data.is_running) {
        log.result(`后端已启动，端口: ${st2.data.port}`);
        recordPass('启动后端', `端口: ${st2.data.port}`);
      } else {
        recordFail('启动后端', '启动后状态检查失败');
      }
    } else {
      recordFail('启动后端', start.error);
    }
  }

  // 测试 6: 获取实例列表
  log.info('\n测试 6: 获取实例列表...');
  const insts = await test('get_instances');
  if (insts.success) {
    log.result(`实例列表: ${JSON.stringify(insts.data, null, 2)}`);
    recordPass('获取实例列表');
  } else {
    recordFail('获取实例列表', insts.error);
  }

  // 打印总结
  log.header('\n' + '='.repeat(60));
  log.header('测试总结');
  log.header('='.repeat(60));
  console.log(`  %c通过: ${results.passed.length}`, 'color: #10b981; font-weight: bold');
  console.log(`  %c失败: ${results.failed.length}`, 'color: #ef4444; font-weight: bold');
  console.log(`  %c跳过: ${results.skipped.length}`, 'color: #f59e0b; font-weight: bold');
  console.log(`  总计: ${results.passed.length + results.failed.length + results.skipped.length}\n`);

  if (results.failed.length > 0) {
    log.error('失败的测试:');
    results.failed.forEach(f => console.log(`  - ${f.name}: ${f.error}`));
  }

  if (results.skipped.length > 0) {
    log.warn('跳过的测试:');
    results.skipped.forEach(s => console.log(`  - ${s.name}: ${s.reason}`));
  }

  log.info('\n测试完成！');

  return results;
})();
