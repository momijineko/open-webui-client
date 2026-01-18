/**
 * Tauri v2 Automated Test Script - Browser Console Version
 *
 * Run this script in the Tauri window developer console
 *
 * Usage:
 * 1. Press F12 in the Tauri window to open the developer console
 * 2. Copy and paste the following code
 * 3. Press Enter to execute
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
    log.result(`Error: ${error}`);
  }

  function recordSkip(name, reason) {
    results.skipped.push({ name, reason });
    log.warn(`${name} - ${reason}`);
  }

  log.header('='.repeat(60));
  log.header('TAURI v2 AUTOMATED TEST');
  log.header('='.repeat(60));

  // Detect Tauri environment
  log.info('\nDetecting Tauri environment...');
  const hasTauri = typeof window !== 'undefined' && (
    '__TAURI__' in window || '__TAURI_INTERNALS__' in window
  );

  if (!hasTauri) {
    recordFail('Tauri Environment', 'Tauri not detected');
    log.error('Please run this test in the Tauri desktop application!');
    return;
  }
  recordPass('Tauri Environment', 'Detected');

  // Try multiple ways to get invoke function
  let invoke = null;

  // Method 1: Through __TAURI_INTERNALS__ (Tauri v2)
  if (window.__TAURI_INTERNALS__) {
    log.result('Found __TAURI_INTERNALS__');
    const internals = window.__TAURI_INTERNALS__;

    // Try different paths
    if (internals.core?.invoke) {
      invoke = internals.core.invoke;
      log.result('Using: __TAURI_INTERNALS__.core.invoke');
    } else if (internals.invoke) {
      invoke = internals.invoke;
      log.result('Using: __TAURI_INTERNALS__.invoke');
    } else if (internals.app?.invoke) {
      invoke = internals.app.invoke;
      log.result('Using: __TAURI_INTERNALS__.app.invoke');
    }
  }

  // Method 2: Through __TAURI__ (Tauri v1/v2)
  if (!invoke && window.__TAURI__) {
    log.result('Found __TAURI__');
    const tauri = window.__TAURI__;

    if (tauri.core?.invoke) {
      invoke = tauri.core.invoke;
      log.result('Using: __TAURI__.core.invoke');
    } else if (tauri.invoke) {
      invoke = tauri.invoke;
      log.result('Using: __TAURI__.invoke');
    } else if (tauri.tauri?.invoke) {
      invoke = tauri.tauri.invoke;
      log.result('Using: __TAURI__.tauri.invoke');
    }
  }

  // Method 3: Check global invoke
  if (!invoke && typeof window.invoke === 'function') {
    invoke = window.invoke;
    log.result('Using: window.invoke');
  }

  if (!invoke) {
    recordFail('invoke function', 'Cannot get invoke function');
    log.error('\nAvailable global objects:');
    log.result('window.__TAURI__: ' + typeof window.__TAURI__);
    log.result('window.__TAURI_INTERNALS__: ' + typeof window.__TAURI_INTERNALS__);

    // Try to print object structure
    if (window.__TAURI_INTERNALS__) {
      log.result('\n__TAURI_INTERNALS__ structure:');
      console.dir(window.__TAURI_INTERNALS__);
    }
    if (window.__TAURI__) {
      log.result('\n__TAURI__ structure:');
      console.dir(window.__TAURI__);
    }
    return;
  }

  recordPass('invoke function', 'Obtained');

  // Test invocation function
  async function test(cmd, args = {}) {
    try {
      const data = await invoke(cmd, args);
      return { success: true, data };
    } catch (error) {
      return { success: false, error: String(error) };
    }
  }

  // Test 1: Get application config
  log.info('\nTest 1: Get application config...');
  const cfg = await test('get_app_config');
  if (cfg.success) {
    log.result(`Config: ${JSON.stringify(cfg.data, null, 2)}`);
    recordPass('Get Application Config');
  } else {
    recordFail('Get Application Config', cfg.error);
  }

  // Test 2: Check backend installation
  log.info('\nTest 2: Check backend installation...');
  const inst = await test('check_backend_installation');
  if (inst.success) {
    log.result(`Installation Status: ${JSON.stringify(inst.data, null, 2)}`);
    recordPass('Check Backend Installation');
  } else {
    recordFail('Check Backend Installation', inst.error);
  }

  // Test 3: Check backend status
  log.info('\nTest 3: Check backend status...');
  const st = await test('check_backend_status');
  if (st.success) {
    log.result(`Running Status: ${JSON.stringify(st.data, null, 2)}`);
    recordPass('Check Backend Status');
  } else {
    recordFail('Check Backend Status', st.error);
  }

  // Test 4: Get download directory
  log.info('\nTest 4: Get download directory...');
  const dir = await test('get_download_dir_path');
  if (dir.success) {
    log.result(`Download Directory: ${dir.data}`);
    recordPass('Get Download Directory');
  } else {
    recordFail('Get Download Directory', dir.error);
  }

  // Test 5: Start backend
  log.info('\nTest 5: Start backend...');
  if (st.success && st.data.is_running) {
    log.result('Backend is already running, skip start test');
    recordSkip('Start Backend', 'Backend is already running');
  } else {
    const start = await test('start_backend');
    if (start.success) {
      log.result(`Start Result: ${start.data}`);
      log.info('Waiting 3 seconds for backend to start...');
      await new Promise(r => setTimeout(r, 3000));
      const st2 = await test('check_backend_status');
      if (st2.success && st2.data.is_running) {
        log.result(`Backend started, port: ${st2.data.port}`);
        recordPass('Start Backend', `Port: ${st2.data.port}`);
      } else {
        recordFail('Start Backend', 'Status check failed after starting');
      }
    } else {
      recordFail('Start Backend', start.error);
    }
  }

  // Test 6: Get instance list
  log.info('\nTest 6: Get instance list...');
  const insts = await test('get_instances');
  if (insts.success) {
    log.result(`Instance List: ${JSON.stringify(insts.data, null, 2)}`);
    recordPass('Get Instance List');
  } else {
    recordFail('Get Instance List', insts.error);
  }

  // Print summary
  log.header('\n' + '='.repeat(60));
  log.header('Test Summary');
  log.header('='.repeat(60));
  console.log(`  %cPassed: ${results.passed.length}`, 'color: #10b981; font-weight: bold');
  console.log(`  %cFailed: ${results.failed.length}`, 'color: #ef4444; font-weight: bold');
  console.log(`  %cSkipped: ${results.skipped.length}`, 'color: #f59e0b; font-weight: bold');
  console.log(`  Total: ${results.passed.length + results.failed.length + results.skipped.length}\n`);

  if (results.failed.length > 0) {
    log.error('Failed tests:');
    results.failed.forEach(f => console.log(`  - ${f.name}: ${f.error}`));
  }

  if (results.skipped.length > 0) {
    log.warn('Skipped tests:');
    results.skipped.forEach(s => console.log(`  - ${s.name}: ${s.reason}`));
  }

  log.info('\nTest completed!');

  return results;
})();
