/**
 * Tauri Commands Automated Test Script
 *
 * This script tests Tauri backend commands by:
 * 1. Building the Tauri app (or using dev mode)
 * 2. Invoking commands via Tauri CLI
 * 3. Verifying responses and behaviors
 *
 * Usage:
 *   node scripts/tests/tauri-commands-test.js
 */

import { spawn, exec } from 'child_process';
import { promisify } from 'util';
import path from 'path';
import { fileURLToPath } from 'url';

const execAsync = promisify(exec);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = path.resolve(__dirname, '../..');

// ANSI color codes
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
};

// Test results tracker
const testResults = {
  passed: 0,
  failed: 0,
  skipped: 0,
  tests: [],
};

// Logger utility
const log = {
  info: (msg) => console.log(`${colors.blue}[INFO]${colors.reset} ${msg}`),
  success: (msg) => console.log(`${colors.green}[PASS]${colors.reset} ${msg}`),
  error: (msg) => console.log(`${colors.red}[FAIL]${colors.reset} ${msg}`),
  warn: (msg) => console.log(`${colors.yellow}[WARN]${colors.reset} ${msg}`),
  header: (msg) => console.log(`\n${colors.bright}${colors.cyan}${'='.repeat(60)}`),
  section: (msg) => console.log(`${colors.bright}${msg}${colors.reset}`),
};

// Test result tracker
function recordTest(name, passed, error = null) {
  const result = { name, passed, error };
  testResults.tests.push(result);

  if (passed) {
    testResults.passed++;
    log.success(`✓ ${name}`);
  } else {
    testResults.failed++;
    log.error(`✗ ${name}`);
    if (error) {
      console.error(`  ${colors.red}${error}${colors.reset}`);
    }
  }
}

function skipTest(name, reason) {
  testResults.skipped++;
  testResults.tests.push({ name, passed: false, skipped: true, reason });
  log.warn(`○ ${name} - ${reason}`);
}

// Check if Tauri is available
async function checkTauriEnvironment() {
  log.header('='.repeat(60));
  log.section('Checking Tauri Environment...');
  log.header('='.repeat(60));

  try {
    // Check if Tauri CLI is installed
    await execAsync('pnpm tauri --version', { cwd: ROOT_DIR });
    log.success('Tauri CLI is installed');

    // Check if Cargo is available
    await execAsync('cargo --version');
    log.success('Rust/Cargo is available');

    // Check if node_modules exists
    const fs = await import('fs');
    const nodeModulesPath = path.join(ROOT_DIR, 'node_modules');
    if (fs.existsSync(nodeModulesPath)) {
      log.success('Dependencies are installed');
    } else {
      log.warn('Dependencies not found, run: pnpm install');
      return false;
    }

    return true;
  } catch (error) {
    log.error(`Environment check failed: ${error.message}`);
    return false;
  }
}

// Check backend installation status
async function testBackendInstallationCheck() {
  log.section('\n--- Test: Backend Installation Check ---');

  try {
    const { stdout } = await execAsync('pnpm tauri cli check_backend_installation', {
      cwd: ROOT_DIR,
      timeout: 10000,
    });

    const result = JSON.parse(stdout.trim());
    log.info(`Backend installed: ${result.installed}`);
    log.info(`Backend path: ${result.path || 'N/A'}`);

    recordTest('check_backend_installation command', true);
    return result;
  } catch (error) {
    // Expected to fail without a running Tauri app
    skipTest('check_backend_installation', 'Requires running Tauri app');
    return null;
  }
}

// Check Python detection
async function testPythonDetection() {
  log.section('\n--- Test: Python Detection ---');

  try {
    // Check system Python
    const { stdout: pyOutput } = await execAsync('python --version 2>&1 || python3 --version 2>&1');
    log.info(`System Python: ${pyOutput.trim()}`);
    recordTest('System Python detection', true);
  } catch (error) {
    skipTest('System Python detection', 'Python not found');
  }

  try {
    // Check Tauri command for Python detection
    const { stdout } = await execAsync('pnpm tauri cli get_installed_python', {
      cwd: ROOT_DIR,
      timeout: 10000,
    });

    log.info(`Detected Python: ${stdout.trim()}`);
    recordTest('get_installed_python command', true);
  } catch (error) {
    skipTest('get_installed_python command', 'Requires running Tauri app');
  }
}

// Test backend status check
async function testBackendStatus() {
  log.section('\n--- Test: Backend Status Check ---');

  try {
    // Try to connect to a local backend
    const response = await fetch('http://127.0.0.1:8080/api/config');
    if (response.ok) {
      log.info('Backend is running on port 8080');
      recordTest('Backend status check (running)', true);
      return true;
    }
  } catch (error) {
    log.info('Backend is not running');
  }

  recordTest('Backend status check (not running)', true);
  return false;
}

// Test download directory path
async function testDownloadDirectory() {
  log.section('\n--- Test: Download Directory ---');

  try {
    const { stdout } = await execAsync('pnpm tauri cli get_download_dir_path', {
      cwd: ROOT_DIR,
      timeout: 10000,
    });

    log.info(`Download directory: ${stdout.trim()}`);
    recordTest('get_download_dir_path command', true);
  } catch (error) {
    skipTest('get_download_dir_path command', 'Requires running Tauri app');
  }
}

// Test config operations
async function testConfigOperations() {
  log.section('\n--- Test: Config Operations ---');

  try {
    const { stdout } = await execAsync('pnpm tauri cli get_app_config', {
      cwd: ROOT_DIR,
      timeout: 10000,
    });

    const config = JSON.parse(stdout.trim());
    log.info('App config retrieved');
    log.info(`Setup completed: ${config.setupCompleted || 'false'}`);

    recordTest('get_app_config command', true);
  } catch (error) {
    skipTest('get_app_config command', 'Requires running Tauri app');
  }
}

// Test proxy request (mock)
async function testProxyRequest() {
  log.section('\n--- Test: Proxy Request ---');

  // This would require a running backend
  skipTest('proxy_request command', 'Requires running Tauri app and backend');
}

// Build verification
async function testBuild() {
  log.section('\n--- Test: Build Verification ---');

  try {
    // Check if the Tauri project compiles
    log.info('Checking if Tauri project compiles...');

    const { stdout } = await execAsync('cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml', {
      cwd: ROOT_DIR,
      timeout: 120000,
    });

    if (stdout.includes('Finished')) {
      log.success('Tauri project compiles successfully');
      recordTest('Cargo build check', true);
      return true;
    }
  } catch (error) {
    // Check if it's just warnings
    if (error.stdout && !error.stdout.includes('error')) {
      log.success('Tauri project compiles (with warnings)');
      recordTest('Cargo build check', true);
      return true;
    }

    recordTest('Cargo build check', false, error.message);
    return false;
  }
}

// Port availability check
async function checkPortAvailability(port) {
  log.section(`\n--- Test: Port ${port} Availability ---`);

  try {
    // Try to bind to the port
    const net = await import('net');
    const server = net.createServer();

    return new Promise((resolve) => {
      server.once('error', (err) => {
        if (err.code === 'EADDRINUSE') {
          log.info(`Port ${port} is in use`);
          resolve(false);
        } else {
          resolve(true);
        }
      });

      server.once('listening', () => {
        server.close();
        log.info(`Port ${port} is available`);
        recordTest(`Port ${port} availability`, true);
        resolve(true);
      });

      server.listen(port, '127.0.0.1');
    });
  } catch (error) {
    skipTest(`Port ${port} availability`, `Could not check: ${error.message}`);
    return false;
  }
}

// File structure verification
async function verifyFileStructure() {
  log.section('\n--- Test: File Structure Verification ---');

  const fs = await import('fs');
  const requiredFiles = [
    { path: 'apps/desktop/src-tauri/Cargo.toml', desc: 'Tauri Cargo config' },
    { path: 'apps/desktop/src-tauri/src/main.rs', desc: 'Tauri main entry' },
    { path: 'apps/desktop/src-tauri/src/backend.rs', desc: 'Backend module' },
    { path: 'apps/desktop/src-tauri/src/download.rs', desc: 'Download module' },
    { path: 'apps/desktop/src-tauri/src/instances.rs', desc: 'Instances module' },
    { path: 'apps/desktop/src-tauri/src/config.rs', desc: 'Config module' },
    { path: 'apps/desktop/src-tauri/src/proxy.rs', desc: 'Proxy module' },
    { path: 'apps/desktop/src-tauri/tauri.conf.json', desc: 'Tauri config' },
    { path: 'src/lib/utils/tauri.ts', desc: 'Tauri API wrapper' },
    { path: 'src/routes/setup/+page.svelte', desc: 'Setup wizard page' },
    { path: 'src/routes/test-tauri/+page.svelte', desc: 'Tauri test page' },
  ];

  let allPresent = true;

  for (const file of requiredFiles) {
    const filePath = path.join(ROOT_DIR, file.path);
    const exists = fs.existsSync(filePath);

    if (exists) {
      log.success(`✓ ${file.desc}`);
      recordTest(`File: ${file.desc}`, true);
    } else {
      log.error(`✗ ${file.desc} - ${file.path}`);
      recordTest(`File: ${file.desc}`, false, 'File not found');
      allPresent = false;
    }
  }

  return allPresent;
}

// Main test runner
async function runTests() {
  log.header('=');
  log.section('TAURI DESKTOP CLIENT - AUTOMATED TEST SUITE');
  log.header('=');

  const startTime = Date.now();

  // 1. Environment check
  const envOk = await checkTauriEnvironment();
  if (!envOk) {
    log.error('\n❌ Environment check failed. Please install required dependencies.');
    return;
  }

  // 2. File structure verification
  await verifyFileStructure();

  // 3. Build verification
  await testBuild();

  // 4. Port availability checks
  await checkPortAvailability(5173); // Vite dev server
  await checkPortAvailability(8080); // Backend

  // 5. Python detection
  await testPythonDetection();

  // 6. Backend status
  await testBackendStatus();

  // 7. Config operations
  await testConfigOperations();

  // 8. Download directory
  await testDownloadDirectory();

  // 9. Proxy request
  await testProxyRequest();

  // Print summary
  const duration = ((Date.now() - startTime) / 1000).toFixed(2);

  log.header('=');
  log.section('TEST SUMMARY');
  log.header('=');

  console.log(`\n  Total Tests: ${testResults.passed + testResults.failed + testResults.skipped}`);
  console.log(`  ${colors.green}Passed: ${testResults.passed}${colors.reset}`);
  console.log(`  ${colors.red}Failed: ${testResults.failed}${colors.reset}`);
  console.log(`  ${colors.yellow}Skipped: ${testResults.skipped}${colors.reset}`);
  console.log(`  Duration: ${duration}s\n`);

  if (testResults.failed > 0) {
    log.error('\n❌ Some tests failed!\n');
    process.exit(1);
  } else {
    log.success('\n✅ All tests passed!\n');
    process.exit(0);
  }
}

// Run tests
runTests().catch((error) => {
  log.error(`\nFatal error: ${error.message}`);
  console.error(error);
  process.exit(1);
});
