#!/usr/bin/env bash
# NEOLAND Resource Monitoring During Load Tests
# Monitors CPU, memory, network, and disk usage

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
INTERVAL="${INTERVAL:-2}"  # Sampling interval in seconds
DURATION="${DURATION:-60}" # Total monitoring duration
PROCESS_NAME="${PROCESS_NAME:-neoland}"
REPORTS_DIR="${REPORTS_DIR:-target/load-reports}"
OUTPUT_FILE="${REPORTS_DIR}/resource_monitor_$(date +%Y%m%d_%H%M%S).csv"

echo -e "${BLUE}╔════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   NEOLAND Resource Monitoring              ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo -e "  Process:       ${PROCESS_NAME}"
echo -e "  Interval:      ${INTERVAL}s"
echo -e "  Duration:      ${DURATION}s"
echo -e "  Output:        ${OUTPUT_FILE}"
echo ""

# Create reports directory
mkdir -p "${REPORTS_DIR}"

# Check if process is running
PID=$(pgrep -f "${PROCESS_NAME}" | head -1)
if [ -z "$PID" ]; then
    echo -e "${RED}❌ Process '${PROCESS_NAME}' not found${NC}"
    echo -e "${YELLOW}Start the server first: cargo run --release --bin neoland -- server${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Found process '${PROCESS_NAME}' (PID: ${PID})${NC}"
echo ""

# CSV header
echo "timestamp,elapsed_sec,cpu_percent,mem_mb,mem_percent,threads,open_files,network_rx_mb,network_tx_mb" > "${OUTPUT_FILE}"

# Initialize counters
START_TIME=$(date +%s)
SAMPLE_COUNT=0

# Get initial network stats
if [ -f "/proc/${PID}/net/dev" ]; then
    INIT_RX=$(cat /proc/net/dev | grep -E "eth0|wlan0|en0" | awk '{print $2}' | head -1)
    INIT_TX=$(cat /proc/net/dev | grep -E "eth0|wlan0|en0" | awk '{print $10}' | head -1)
else
    INIT_RX=0
    INIT_TX=0
fi

echo -e "${CYAN}🔍 Monitoring started... (Ctrl+C to stop early)${NC}"
echo ""
echo -e "${MAGENTA}Time    CPU%   Memory(MB)  Threads  Files  RX(MB)  TX(MB)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Monitoring loop
while true; do
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - START_TIME))

    if [ $ELAPSED -ge $DURATION ]; then
        break
    fi

    # Check if process still exists
    if ! kill -0 $PID 2>/dev/null; then
        echo -e "${RED}❌ Process died during monitoring${NC}"
        break
    fi

    # Get CPU and memory usage
    if command -v ps &> /dev/null; then
        # Linux/macOS compatible
        CPU_PERCENT=$(ps -p $PID -o %cpu= 2>/dev/null | xargs)
        MEM_KB=$(ps -p $PID -o rss= 2>/dev/null | xargs)
        MEM_MB=$(echo "scale=2; ${MEM_KB:-0} / 1024" | bc)
        MEM_PERCENT=$(ps -p $PID -o %mem= 2>/dev/null | xargs)
    else
        CPU_PERCENT="N/A"
        MEM_MB="N/A"
        MEM_PERCENT="N/A"
    fi

    # Get thread count
    if [ -d "/proc/${PID}/task" ]; then
        THREADS=$(ls -1 /proc/${PID}/task | wc -l)
    else
        THREADS="N/A"
    fi

    # Get open files count
    if command -v lsof &> /dev/null; then
        OPEN_FILES=$(lsof -p $PID 2>/dev/null | wc -l)
    elif [ -d "/proc/${PID}/fd" ]; then
        OPEN_FILES=$(ls -1 /proc/${PID}/fd 2>/dev/null | wc -l)
    else
        OPEN_FILES="N/A"
    fi

    # Get network stats
    if [ -f "/proc/net/dev" ]; then
        CURR_RX=$(cat /proc/net/dev | grep -E "eth0|wlan0|en0" | awk '{print $2}' | head -1)
        CURR_TX=$(cat /proc/net/dev | grep -E "eth0|wlan0|en0" | awk '{print $10}' | head -1)
        RX_MB=$(echo "scale=2; (${CURR_RX:-$INIT_RX} - ${INIT_RX}) / 1048576" | bc)
        TX_MB=$(echo "scale=2; (${CURR_TX:-$INIT_TX} - ${INIT_TX}) / 1048576" | bc)
    else
        RX_MB="N/A"
        TX_MB="N/A"
    fi

    # Format timestamp
    TIMESTAMP=$(date +"%H:%M:%S")

    # Write to CSV
    echo "${TIMESTAMP},${ELAPSED},${CPU_PERCENT},${MEM_MB},${MEM_PERCENT},${THREADS},${OPEN_FILES},${RX_MB},${TX_MB}" >> "${OUTPUT_FILE}"

    # Display to console (formatted)
    printf "${CYAN}%s${NC}  %6s  %10s  %7s  %5s  %6s  %6s\n" \
        "${TIMESTAMP}" \
        "${CPU_PERCENT}" \
        "${MEM_MB}" \
        "${THREADS}" \
        "${OPEN_FILES}" \
        "${RX_MB}" \
        "${TX_MB}"

    SAMPLE_COUNT=$((SAMPLE_COUNT + 1))
    sleep $INTERVAL
done

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Monitoring complete${NC}"
echo ""

# Generate summary statistics
if command -v awk &> /dev/null; then
    echo -e "${MAGENTA}📊 Summary Statistics:${NC}"
    echo ""

    # CPU stats
    CPU_AVG=$(awk -F',' 'NR>1 && $3!="N/A" {sum+=$3; count++} END {if(count>0) printf "%.2f", sum/count; else print "N/A"}' "${OUTPUT_FILE}")
    CPU_MAX=$(awk -F',' 'NR>1 && $3!="N/A" {if($3>max || max=="") max=$3} END {printf "%.2f", max}' "${OUTPUT_FILE}")

    echo -e "   ${CYAN}CPU Usage:${NC}"
    echo -e "     Average:  ${CPU_AVG}%"
    echo -e "     Peak:     ${CPU_MAX}%"
    echo ""

    # Memory stats
    MEM_AVG=$(awk -F',' 'NR>1 && $4!="N/A" {sum+=$4; count++} END {if(count>0) printf "%.2f", sum/count; else print "N/A"}' "${OUTPUT_FILE}")
    MEM_MAX=$(awk -F',' 'NR>1 && $4!="N/A" {if($4>max || max=="") max=$4} END {printf "%.2f", max}' "${OUTPUT_FILE}")

    echo -e "   ${CYAN}Memory Usage:${NC}"
    echo -e "     Average:  ${MEM_AVG} MB"
    echo -e "     Peak:     ${MEM_MAX} MB"
    echo ""

    # Thread stats
    THREADS_AVG=$(awk -F',' 'NR>1 && $6!="N/A" {sum+=$6; count++} END {if(count>0) printf "%.0f", sum/count; else print "N/A"}' "${OUTPUT_FILE}")
    THREADS_MAX=$(awk -F',' 'NR>1 && $6!="N/A" {if($6>max || max=="") max=$6} END {printf "%.0f", max}' "${OUTPUT_FILE}")

    echo -e "   ${CYAN}Threads:${NC}"
    echo -e "     Average:  ${THREADS_AVG}"
    echo -e "     Peak:     ${THREADS_MAX}"
    echo ""

    # Network stats
    TOTAL_RX=$(awk -F',' 'END {if($8!="N/A") printf "%.2f", $8; else print "N/A"}' "${OUTPUT_FILE}")
    TOTAL_TX=$(awk -F',' 'END {if($9!="N/A") printf "%.2f", $9; else print "N/A"}' "${OUTPUT_FILE}")

    echo -e "   ${CYAN}Network Transfer:${NC}"
    echo -e "     Received: ${TOTAL_RX} MB"
    echo -e "     Sent:     ${TOTAL_TX} MB"
    echo ""
fi

echo -e "${MAGENTA}📁 Full report:${NC} ${OUTPUT_FILE}"
echo -e "${YELLOW}💡 Visualize: Import CSV into spreadsheet or gnuplot${NC}"
echo ""

# Check for resource warnings
if [ -n "${MEM_MAX}" ] && (( $(echo "${MEM_MAX} > 2000" | bc -l 2>/dev/null || echo 0) )); then
    echo -e "${YELLOW}⚠️  WARNING: High memory usage detected (${MEM_MAX} MB)${NC}"
fi

if [ -n "${CPU_AVG}" ] && (( $(echo "${CPU_AVG} > 80" | bc -l 2>/dev/null || echo 0) )); then
    echo -e "${YELLOW}⚠️  WARNING: High average CPU usage (${CPU_AVG}%)${NC}"
fi

if [ -n "${THREADS_MAX}" ] && [ "${THREADS_MAX}" != "N/A" ] && [ $THREADS_MAX -gt 1000 ]; then
    echo -e "${YELLOW}⚠️  WARNING: High thread count (${THREADS_MAX})${NC}"
fi

echo ""
echo -e "${GREEN}╔════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   ✅ Resource monitoring complete          ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════╝${NC}"
