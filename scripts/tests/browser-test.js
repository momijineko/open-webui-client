/**
 * Tauri Automated Test Script - Browser Console Version
 *
 * Run this script in the Tauri window developer console
 *
 * Usage:
 * 1. Press F12 in the Tauri window to open the developer console
 * 2. Copy and paste the following code
 * 3. Press Enter to execute
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
    log.result(`Error: ${error}`);
  }

  function recordSkip(name, reason) {
    results.skipped.push({ name, reason });
    log.warn(`${name} - ${reason}`);
  }

  // Check if Tauri is available
  function isTauriAvailable() {
    try {
      return typeof window.__TAURI__ !== 'undefined' ||
             typeof window.__TAURI_INTERNALS__ !== 'undefined';
    } catch {
      return false;
    }
  }

  // Get Tauri invoke function
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
  log.header('TAURI AUTOMATED TEST');
  log.header('='.repeat(60));
  console.log();

  // Test 1: Tauri availability check
  log.section('\n--- Test 1: Tauri Availability Check ---');
  if (isTauriAvailable()) {
    recordPass('Tauri Availability Check', 'Tauri environment is normal');
  } else {
    recordFail('Tauri Availability Check', 'Tauri is not available (non-desktop environment)');
    log.error('Please run this test in the Tauri desktop application!');
    return;
  }

  const invoke = getInvoke();
  if (!invoke) {
    recordFail('Tauri invoke function', 'Cannot get invoke function');
    return;
  }
  recordPass('Tauri invoke function', 'invoke function is available');

  // Test helper function
  async function invokeCommand(cmd, args = {}) {
    try {
      const result = await invoke(cmd, args);
      return { success: true, data: result };
    } catch (error) {
      return { success: false, error: error.message || error.toString() };
    }
  }

  // Test 2: Get application config
  log.section('\n--- Test 2: Get Application Config ---');
  const configResult = await invokeCommand('get_app_config');
  if (configResult.success) {
    log.result(`Config: ${JSON.stringify(configResult.data, null, 2)}`);
    recordPass('Get Application Config');
  } else {
    recordFail('Get Application Config', configResult.error);
  }

  // Test 3: Check backend installation status
  log.section('\n--- Test 3: Check Backend Installation Status ---');
  const installResult = await invokeCommand('check_backend_installation');
  if (installResult.success) {
    log.result(`Installation Status: ${JSON.stringify(installResult.data, null, 2)}`);
    recordPass('Check Backend Installation Status');
  } else {
    recordFail('Check Backend Installation Status', installResult.error);
  }

  // Test 4: Check backend running status
  log.section('\n--- Test 4: Check Backend Running Status ---');
  const statusResult = await invokeCommand('check_backend_status');
  if (statusResult.success) {
    log.result(`Running Status: ${JSON.stringify(statusResult.data, null, 2)}`);
    recordPass('Check Backend Running Status');
  } else {
    recordFail('Check Backend Running Status', statusResult.error);
  }

  // Test 5: Get download directory
  log.section('\n--- Test 5: Get Download Directory ---');
  const dirResult = await invokeCommand('get_download_dir_path');
  if (dirResult.success) {
    log.result(`Download Directory: ${dirResult.data}`);
    recordPass('Get Download Directory');
  } else {
    recordFail('Get Download Directory', dirResult.error);
  }

  // Test 6: Start backend (if not running)
  log.section('\n--- Test 6: Start Backend Test ---');
  if (statusResult.success && statusResult.data.is_running) {
    log.result('Backend is already running, skip start test');
    recordSkip('Start Backend', 'Backend is already running');
  } else {
    log.info('Starting backend...');
    const startResult = await invokeCommand('start_backend');
    if (startResult.success) {
      log.result(`Start Result: ${startResult.data}`);

      // Wait for backend to start
      log.info('Waiting 3 seconds for backend to start...');
      await new Promise(r => setTimeout(r, 3000));

      // Check status after starting
      const newStatusResult = await invokeCommand('check_backend_status');
      if (newStatusResult.success && newStatusResult.data.is_running) {
        log.result(`Backend started, port: ${newStatusResult.data.port}`);
        recordPass('Start Backend', `Port: ${newStatusResult.data.port}`);
      } else {
        recordFail('Start Backend', 'Status check failed after starting');
      }
    } else {
      recordFail('Start Backend', startResult.error);
    }
  }

  // Test 7: Get backend logs
  log.section('\n--- Test 7: Get Backend Logs ---');
  const logsResult = await invokeCommand('get_backend_logs');
  if (logsResult.success) {
    const logs = logsResult.data;
    if (logs && logs.length > 0) {
      log.result(`Recent logs (${logs.length} entries):`);
      logs.slice(-5).forEach(log => log.result(`  ${log}`));
    } else {
      log.result('No logs available');
    }
    recordPass('Get Backend Logs');
  } else {
    recordFail('Get Backend Logs', logsResult.error);
  }

  // Print summary
  log.header('\n' + '='.repeat(60));
  log.header('Test Summary');
  log.header('='.repeat(60));

  console.log(`\n  %cPassed: ${results.passed.length}`, 'color: #10b981; font-weight: bold');
  console.log(`  %cFailed: ${results.failed.length}`, 'color: #ef4444; font-weight: bold');
  console.log(`  %cSkipped: ${results.skipped.length}`, 'color: #f59e0b; font-weight: bold');
  console.log(`  Total: ${results.passed.length + results.failed.length + results.skipped.length}\n`);

  if (results.failed.length > 0) {
    log.error('\nFailed tests:');
    results.failed.forEach((f) => {
      console.log(`  - ${f.name}: ${f.error}`);
    });
  }

  if (results.skipped.length > 0) {
    log.warn('\nSkipped tests:');
    results.skipped.forEach((s) => {
      console.log(`  - ${s.name}: ${s.reason}`);
    });
  }

  log.info('\nTest completed!');

  return results;
})();
