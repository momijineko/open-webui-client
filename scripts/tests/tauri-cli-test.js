/**
 * Tauri CLI Automated Test Script
 *
 * Test Tauri command functionality directly through command line
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

// Test: Tauri availability check
async function testTauriAvailability() {
  log.test('Tauri Availability Check');
  try {
    // Try to run a simple Tauri command
    const output = await invokeTauriCommand('help');
    if (output.includes('Usage:')) {
      recordPass('Tauri Availability Check', 'Tauri CLI is available');
      return true;
    }
  } catch (error) {
    recordFail('Tauri Availability Check', error.message);
    return false;
  }
}

// Test: Get application config
async function testGetAppConfig() {
  log.test('Get Application Config');
  try {
    const output = await invokeTauriCommand('get_app_config');
    log.info(`Config Output: ${output.substring(0, 100)}...`);
    recordPass('Get Application Config');
    return JSON.parse(output);
  } catch (error) {
    recordFail('Get Application Config', error.message);
    return null;
  }
}

// Test: Check backend installation status
async function testBackendInstallation() {
  log.test('Check Backend Installation Status');
  try {
    const output = await invokeTauriCommand('check_backend_installation');
    log.info(`Backend Status: ${output}`);
    recordPass('Check Backend Installation Status');
    return JSON.parse(output);
  } catch (error) {
    recordFail('Check Backend Installation Status', error.message);
    return null;
  }
}

// Test: Check backend running status
async function testBackendStatus() {
  log.test('Check Backend Running Status');
  try {
    const output = await invokeTauriCommand('check_backend_status');
    log.info(`Status Output: ${output}`);
    recordPass('Check Backend Running Status');
    return JSON.parse(output);
  } catch (error) {
    recordFail('Check Backend Running Status', error.message);
    return null;
  }
}

// Test: Get download directory
async function testGetDownloadDir() {
  log.test('Get Download Directory');
  try {
    const output = await invokeTauriCommand('get_download_dir_path');
    log.info(`Download Directory: ${output}`);
    recordPass('Get Download Directory');
    return output;
  } catch (error) {
    recordFail('Get Download Directory', error.message);
    return null;
  }
}

// Test: Start backend
async function testStartBackend() {
  log.test('Start Backend (Test)');
  try {
    // First check if backend is already running
    const statusOutput = await invokeTauriCommand('check_backend_status');
    const status = JSON.parse(statusOutput);

    if (status.is_running) {
      log.info('Backend is already running, skip start test');
      recordPass('Start Backend', 'Backend is already running');
      return { skipped: true, alreadyRunning: true };
    }

    // Try to start backend
    const output = await invokeTauriCommand('start_backend');
    log.info(`Start Output: ${output}`);

    // Wait a few seconds for backend to start
    await new Promise(resolve => setTimeout(resolve, 3000));

    // Check status again
    const newStatusOutput = await invokeTauriCommand('check_backend_status');
    const newStatus = JSON.parse(newStatusOutput);

    if (newStatus.is_running) {
      recordPass('Start Backend', `Backend started, port: ${newStatus.port}`);
      return { success: true, port: newStatus.port };
    } else {
      recordFail('Start Backend', 'Status check failed after starting');
      return { success: false };
    }
  } catch (error) {
    recordFail('Start Backend', error.message);
    return { success: false };
  }
}

// Test: Stop backend
async function testStopBackend() {
  log.test('Stop Backend (Test)');
  try {
    const output = await invokeTauriCommand('stop_backend');
    log.info(`Stop Output: ${output}`);

    // Wait a few seconds for backend to stop
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Check status
    const statusOutput = await invokeTauriCommand('check_backend_status');
    const status = JSON.parse(statusOutput);

    if (!status.is_running) {
      recordPass('Stop Backend');
      return true;
    } else {
      recordFail('Stop Backend', 'Backend is still running');
      return false;
    }
  } catch (error) {
    recordFail('Stop Backend', error.message);
    return false;
  }
}

// Print summary
function printSummary() {
  log.header('=');
  log.section('Test Summary');
  log.header('=');

  console.log(`\n  ${colors.green}Passed: ${results.passed.length}${colors.reset}`);
  console.log(`  ${colors.red}Failed: ${results.failed.length}${colors.reset}`);
  console.log(`  ${colors.yellow}Skipped: ${results.skipped.length}${colors.reset}`);
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
}

// Main test runner
async function runTests() {
  log.header('=');
  log.section('TAURI COMMAND LINE AUTOMATED TEST');
  log.header('=');

  log.info('Starting Tauri command tests...\n');

  // Run tests
  await testTauriAvailability();
  await testGetAppConfig();
  await testBackendInstallation();
  await testBackendStatus();
  await testGetDownloadDir();

  // Backend start/stop tests (optional, requires more time)
  log.section('\nTest backend start/stop functionality?');
  log.info('(This will start the actual Python backend process)');
  log.info('Skipping this test because it requires more time');

  recordSkip('Start Backend Test', 'Requires manual confirmation');
  recordSkip('Stop Backend Test', 'Requires manual confirmation');

  // Print summary
  printSummary();

  // Return exit code
  const exitCode = results.failed.length > 0 ? 1 : 0;
  process.exit(exitCode);
}

// Run tests
runTests().catch((err) => {
  log.error(`\nFatal Error: ${err.message}`);
  console.error(err);
  process.exit(1);
});
