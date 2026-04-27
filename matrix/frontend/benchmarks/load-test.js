#!/usr/bin/env node

/**
 * Advanced Load Testing for Dev Assistant Hub
 * Simulates realistic developer workflows with configurable parameters
 */

const http = require("http")
const https = require("https")
const { performance } = require("perf_hooks")
const fs = require("fs")
const path = require("path")

// Configuration
const CONFIG = {
  baseUrl: process.env.BASE_URL || "http://localhost:3006",
  concurrency: Number.parseInt(process.env.CONCURRENCY) || 10,
  duration: Number.parseInt(process.env.DURATION) || 30000, // 30 seconds
  rampUpTime: Number.parseInt(process.env.RAMP_UP) || 5000, // 5 seconds
  outputDir: "benchmarks/results",
  endpoints: [
    { path: "/", weight: 30, name: "Homepage" },
    { path: "/api/health", weight: 20, name: "Health Check" },
    {
      path: "/api/code-analyzer",
      weight: 15,
      name: "Code Analyzer",
      method: "POST",
      body: { code: 'console.log("test");' },
    },
    { path: "/api/llm-playground", weight: 15, name: "LLM Playground" },
    { path: "/agent-orchestra", weight: 10, name: "Agent Orchestra" },
    { path: "/prototype-preview", weight: 10, name: "Prototype Preview" },
  ],
}

// Statistics tracking
const stats = {
  requests: 0,
  responses: 0,
  errors: 0,
  responseTimes: [],
  errorTypes: {},
  statusCodes: {},
  startTime: 0,
  endTime: 0,
  concurrentUsers: 0,
  maxConcurrentUsers: 0,
}

// Utility functions
function log(message, level = "INFO") {
  const timestamp = new Date().toISOString()
  const colors = {
    INFO: "\x1b[36m",
    SUCCESS: "\x1b[32m",
    WARNING: "\x1b[33m",
    ERROR: "\x1b[31m",
    RESET: "\x1b[0m",
  }
  console.log(`${colors[level]}[${timestamp}] ${level}: ${message}${colors.RESET}`)
}

function makeRequest(endpoint) {
  return new Promise((resolve) => {
    const startTime = performance.now()
    const url = new URL(endpoint.path, CONFIG.baseUrl)
    const isHttps = url.protocol === "https:"
    const client = isHttps ? https : http

    const options = {
      hostname: url.hostname,
      port: url.port || (isHttps ? 443 : 80),
      path: url.pathname + url.search,
      method: endpoint.method || "GET",
      headers: {
        "User-Agent": "DevAssistantHub-LoadTest/1.0",
        Accept: "application/json, text/html, */*",
        Connection: "keep-alive",
      },
    }

    if (endpoint.body) {
      options.headers["Content-Type"] = "application/json"
      options.headers["Content-Length"] = Buffer.byteLength(JSON.stringify(endpoint.body))
    }

    stats.requests++
    stats.concurrentUsers++
    stats.maxConcurrentUsers = Math.max(stats.maxConcurrentUsers, stats.concurrentUsers)

    const req = client.request(options, (res) => {
      let data = ""

      res.on("data", (chunk) => {
        data += chunk
      })

      res.on("end", () => {
        const endTime = performance.now()
        const responseTime = endTime - startTime

        stats.responses++
        stats.responseTimes.push(responseTime)
        stats.statusCodes[res.statusCode] = (stats.statusCodes[res.statusCode] || 0) + 1
        stats.concurrentUsers--

        resolve({
          success: true,
          statusCode: res.statusCode,
          responseTime,
          dataLength: data.length,
        })
      })
    })

    req.on("error", (error) => {
      const endTime = performance.now()
      const responseTime = endTime - startTime

      stats.errors++
      stats.errorTypes[error.code || "UNKNOWN"] = (stats.errorTypes[error.code || "UNKNOWN"] || 0) + 1
      stats.concurrentUsers--

      resolve({
        success: false,
        error: error.message,
        responseTime,
      })
    })

    req.on("timeout", () => {
      req.destroy()
      stats.errors++
      stats.errorTypes["TIMEOUT"] = (stats.errorTypes["TIMEOUT"] || 0) + 1
      stats.concurrentUsers--

      resolve({
        success: false,
        error: "Request timeout",
        responseTime: performance.now() - startTime,
      })
    })

    req.setTimeout(10000) // 10 second timeout

    if (endpoint.body) {
      req.write(JSON.stringify(endpoint.body))
    }

    req.end()
  })
}

function selectEndpoint() {
  const totalWeight = CONFIG.endpoints.reduce((sum, ep) => sum + ep.weight, 0)
  let random = Math.random() * totalWeight

  for (const endpoint of CONFIG.endpoints) {
    random -= endpoint.weight
    if (random <= 0) {
      return endpoint
    }
  }

  return CONFIG.endpoints[0] // Fallback
}

function calculatePercentile(arr, percentile) {
  const sorted = [...arr].sort((a, b) => a - b)
  const index = Math.ceil((percentile / 100) * sorted.length) - 1
  return sorted[index] || 0
}

function generateReport() {
  const duration = (stats.endTime - stats.startTime) / 1000
  const rps = stats.responses / duration
  const errorRate = (stats.errors / stats.requests) * 100

  const report = {
    timestamp: new Date().toISOString(),
    configuration: {
      baseUrl: CONFIG.baseUrl,
      concurrency: CONFIG.concurrency,
      duration: CONFIG.duration,
      rampUpTime: CONFIG.rampUpTime,
    },
    summary: {
      totalRequests: stats.requests,
      successfulResponses: stats.responses,
      errors: stats.errors,
      errorRate: Number.parseFloat(errorRate.toFixed(2)),
      duration: Number.parseFloat(duration.toFixed(2)),
      requestsPerSecond: Number.parseFloat(rps.toFixed(2)),
      maxConcurrentUsers: stats.maxConcurrentUsers,
    },
    responseTime: {
      min: Math.min(...stats.responseTimes),
      max: Math.max(...stats.responseTimes),
      mean: stats.responseTimes.reduce((a, b) => a + b, 0) / stats.responseTimes.length,
      p50: calculatePercentile(stats.responseTimes, 50),
      p95: calculatePercentile(stats.responseTimes, 95),
      p99: calculatePercentile(stats.responseTimes, 99),
    },
    statusCodes: stats.statusCodes,
    errorTypes: stats.errorTypes,
    endpoints: CONFIG.endpoints.map((ep) => ({
      name: ep.name,
      path: ep.path,
      weight: ep.weight,
    })),
  }

  // Ensure output directory exists
  if (!fs.existsSync(CONFIG.outputDir)) {
    fs.mkdirSync(CONFIG.outputDir, { recursive: true })
  }

  // Save detailed results
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-")
  const filename = path.join(CONFIG.outputDir, `load-test-${timestamp}.json`)
  fs.writeFileSync(filename, JSON.stringify(report, null, 2))

  return { report, filename }
}

async function runLoadTest() {
  log("🚀 Starting Advanced Load Test for Dev Assistant Hub")
  log(`Configuration: ${CONFIG.concurrency} concurrent users, ${CONFIG.duration}ms duration`)
  log(`Target: ${CONFIG.baseUrl}`)

  stats.startTime = performance.now()
  const testEndTime = stats.startTime + CONFIG.duration
  const rampUpEndTime = stats.startTime + CONFIG.rampUpTime

  const workers = []
  let activeWorkers = 0

  // Ramp-up phase
  log("📈 Ramp-up phase starting...")
  const rampUpInterval = CONFIG.rampUpTime / CONFIG.concurrency

  for (let i = 0; i < CONFIG.concurrency; i++) {
    setTimeout(() => {
      const worker = async () => {
        activeWorkers++
        while (performance.now() < testEndTime) {
          const endpoint = selectEndpoint()
          try {
            await makeRequest(endpoint)
          } catch (error) {
            log(`Request failed: ${error.message}`, "ERROR")
          }

          // Small delay between requests from same worker
          await new Promise((resolve) => setTimeout(resolve, Math.random() * 100))
        }
        activeWorkers--
      }

      workers.push(worker())
    }, i * rampUpInterval)
  }

  // Progress reporting
  const progressInterval = setInterval(() => {
    const elapsed = (performance.now() - stats.startTime) / 1000
    const remaining = Math.max(0, CONFIG.duration / 1000 - elapsed)
    const rps = stats.responses / elapsed

    log(
      `Progress: ${elapsed.toFixed(1)}s elapsed, ${remaining.toFixed(1)}s remaining | ` +
        `RPS: ${rps.toFixed(1)} | Active: ${activeWorkers} | Errors: ${stats.errors}`,
    )
  }, 2000)

  // Wait for all workers to complete
  await Promise.all(workers)
  clearInterval(progressInterval)

  stats.endTime = performance.now()

  log("✅ Load test completed!")

  // Generate and save report
  const { report, filename } = generateReport()

  // Display summary
  log("📊 Load Test Results Summary:", "SUCCESS")
  log(`Total Requests: ${report.summary.totalRequests}`)
  log(`Successful Responses: ${report.summary.successfulResponses}`)
  log(`Error Rate: ${report.summary.errorRate}%`)
  log(`Requests/Second: ${report.summary.requestsPerSecond}`)
  log(`Response Time P95: ${report.responseTime.p95.toFixed(2)}ms`)
  log(`Response Time P99: ${report.responseTime.p99.toFixed(2)}ms`)
  log(`Max Concurrent Users: ${report.summary.maxConcurrentUsers}`)
  log(`Report saved to: ${filename}`, "SUCCESS")

  return report
}

// CLI argument parsing
if (require.main === module) {
  const args = process.argv.slice(2)

  args.forEach((arg) => {
    const [key, value] = arg.split("=")
    switch (key) {
      case "--concurrency":
        CONFIG.concurrency = Number.parseInt(value)
        break
      case "--duration":
        CONFIG.duration = Number.parseInt(value)
        break
      case "--url":
        CONFIG.baseUrl = value
        break
      case "--ramp-up":
        CONFIG.rampUpTime = Number.parseInt(value)
        break
    }
  })

  runLoadTest().catch((error) => {
    log(`Load test failed: ${error.message}`, "ERROR")
    process.exit(1)
  })
}

module.exports = { runLoadTest, CONFIG }
