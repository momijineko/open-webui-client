/**
 * Tauri Runtime Test Script
 *
 * This script tests Tauri commands while the app is running.
 * It monitors the Tauri dev server output and performs tests.
 *
 * Usage:
 *   node scripts/tests/tauri-runtime-test.js
 */

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { promises as fs } from 'fs';

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
  warn: (msg) => console.log(`${colors.yellow}[WARN]${colors.reset} ${msg}`),
  header: (msg) => console.log(`\n${colors.bright}${colors.cyan}${'='.repeat(60)}`),
  section: (msg) => console.log(`${colors.bright}${msg}${colors.reset}`),
  tauri: (msg) => console.log(`${colors.magenta}[TAURI]${colors.reset} ${msg}`),
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

// Check backend port
async function checkBackendPort(port = 8080) {
  log.section(`\n--- Test: Backend Port ${port} ---`);

  try {
    const response = await fetch(`http://127.0.0.1:${port}/health`);
    if (response.ok) {
      recordPass(`Backend health check (port ${port})`);
      return true;
    }
  } catch (error) {
    // Try alternative endpoint
  }

  try {
    const response = await fetch(`http://127.0.0.1:${port}/api/config`);
    if (response.ok) {
      recordPass(`Backend API check (port ${port})`);
      return true;
    }
  } catch (error) {
    recordFail(`Backend check (port ${port})`, 'Backend not responding');
    return false;
  }
}

// Check Vite dev server
async function checkViteServer(port = 5173) {
  log.section(`\n--- Test: Vite Dev Server ${port} ---`);

  try {
    const response = await fetch(`http://127.0.0.1:${port}/`);
    if (response.ok) {
      const html = await response.text();
      if (html.includes('vite') || html.includes('Open WebUI')) {
        recordPass(`Vite dev server (port ${port})`);
        return true;
      }
    }
  } catch (error) {
    recordFail(`Vite dev server (port ${port})`, error.message);
  }

  return false;
}

// Parse Tauri output for test results
function parseTauriOutput(line) {
  const lower = line.toLowerCase();

  // Check for compilation success
  if (lower.includes('finished') && lower.includes('dev')) {
    recordPass('Tauri compilation', 'Compiled successfully');
  }

  // Check for backend start
  if (lower.includes('backend started') || lower.includes('backend is running')) {
    recordPass('Backend startup');
  }

  // Check for errors
  if (lower.includes('error') && !lower.includes('error[E') && !lower.includes('warning')) {
    if (!lower.includes('error: could not')) {
      log.tauri(line.trim());
    }
  }

  // Check for specific Tauri events
  if (lower.includes('ipc')) {
    log.tauri(line.trim());
  }
}

// Monitor Tauri process
async function monitorTauriProcess() {
  return new Promise((resolve) => {
    log.header('=');
    log.section('STARTING TAURI DEV SERVER');
    log.header('=');

    const tauri = spawn('pnpm', ['tauri:dev'], {
      cwd: ROOT_DIR,
      shell: true,
      env: { ...process.env, NODE_ENV: 'development' },
    });

    let outputBuffer = [];
    let serverReady = false;

    tauri.stdout.on('data', (data) => {
      const lines = data.toString().split('\n');
      lines.forEach((line) => {
        if (line.trim()) {
          outputBuffer.push(line);
          parseTauriOutput(line);

          // Check if server is ready
          if (line.includes('Running') || line.includes('ready') || line.includes('listening')) {
            serverReady = true;
          }
        }
      });
    });

    tauri.stderr.on('data', (data) => {
      const lines = data.toString().split('\n');
      lines.forEach((line) => {
        if (line.trim()) {
          outputBuffer.push(line);
          log.tauri(line.trim());
        }
      });
    });

    // Wait for servers to be ready
    setTimeout(async () => {
      log.section('\n--- Test: Server Availability ---');

      // Check Vite server
      await checkViteServer(5173);

      // Wait a bit more for backend
      await new Promise((r) => setTimeout(r, 3000));

      // Check backend
      await checkBackendPort(8080);

      log.info('\nServers are running. You can now:');
      log.info('  1. Open http://localhost:5173 in your browser');
      log.info('  2. Navigate to /test-tauri to run command tests');
      log.info('  3. Navigate to /setup to test the installation wizard');

      log.section('\n--- Press Ctrl+C to stop the server ---\n');

      // Keep process running
    }, 15000);

    tauri.on('close', (code) => {
      log.info(`\nTauri process exited with code ${code}`);
      resolve();
    });

    // Handle exit
    process.on('SIGINT', () => {
      log.info('\n\nStopping Tauri server...');
      tauri.kill();
      process.exit(0);
    });
  });
}

// Print summary
function printSummary() {
  log.header('=');
  log.section('TEST SUMMARY');
  log.header('=');

  console.log(`\n  ${colors.green}Passed: ${results.passed.length}${colors.reset}`);
  console.log(`  ${colors.red}Failed: ${results.failed.length}${colors.reset}`);
  console.log(`  ${colors.yellow}Skipped: ${results.skipped.length}${colors.reset}\n`);

  if (results.failed.length > 0) {
    log.error('\nFailed tests:');
    results.failed.forEach((f) => {
      console.log(`  - ${f.name}: ${f.error}`);
    });
  }
}

// Main
async function main() {
  log.header('=');
  log.section('TAURI RUNTIME TEST SUITE');
  log.header('=');

  log.info('This script will start the Tauri dev server and monitor its output.');
  log.info('The server will continue running until you press Ctrl+C.\n');

  await monitorTauriProcess();
}

main().catch((err) => {
  log.error(`Fatal error: ${err.message}`);
  console.error(err);
  process.exit(1);
});
