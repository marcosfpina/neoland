#!/usr/bin/env python3
"""
Mission Control System Daemon
Real-time system metrics server via Unix Domain Socket (FastAPI)
"""

import asyncio
import json
import os
import socket
import subprocess
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, Optional, List
from contextlib import asynccontextmanager

import uvicorn
from fastapi import FastAPI, WebSocket, WebSocketDisconnect
from fastapi.responses import JSONResponse

try:
    import psutil
except ImportError:
    psutil = None

# --- Configuration ---
UDS_PATH = "/tmp/mission-control.sock"
DATA_DIR = Path(os.environ.get("DATA_DIR", "/var/lib/mission-control"))
PROC_PATH = Path(os.environ.get("PROC_PATH", "/proc"))
SYS_PATH = Path(os.environ.get("SYS_PATH", "/sys"))


# --- Metrics Collector Class (Preserved & Optimized) ---
class SystemMetricsCollector:
    """Collects real system metrics from /proc, /sys, and system commands"""
    
    def __init__(self):
        self.hostname = socket.gethostname()
        self.proc_path = PROC_PATH
        self.sys_path = SYS_PATH
        
    def read_proc_file(self, path: str) -> Optional[str]:
        try:
            return (self.proc_path / path).read_text()
        except (FileNotFoundError, PermissionError):
            return None
    
    def read_sys_file(self, path: str) -> Optional[str]:
        try:
            return (self.sys_path / path).read_text().strip()
        except (FileNotFoundError, PermissionError):
            return None
    
    def get_cpu_info(self) -> Dict[str, Any]:
        info = {
            "cores": 0, "model": "Unknown", "frequency_mhz": 0,
            "usage_percent": 0, "per_core_usage": [],
            "load_average": [0, 0, 0], "governor": "unknown"
        }
        
        cpuinfo = self.read_proc_file("cpuinfo")
        if cpuinfo:
            info["cores"] = cpuinfo.count("processor")
            for line in cpuinfo.split("\n"):
                if "model name" in line:
                    info["model"] = line.split(":")[1].strip()
                    break
                if "cpu MHz" in line:
                    info["frequency_mhz"] = float(line.split(":")[1].strip())
        
        loadavg = self.read_proc_file("loadavg")
        if loadavg:
            parts = loadavg.split()
            info["load_average"] = [float(parts[0]), float(parts[1]), float(parts[2])]
        
        if psutil:
            info["usage_percent"] = psutil.cpu_percent(interval=None)
            info["per_core_usage"] = psutil.cpu_percent(interval=None, percpu=True)
        
        governor = self.read_sys_file("devices/system/cpu/cpu0/cpufreq/scaling_governor")
        if governor:
            info["governor"] = governor
            
        return info
    
    def get_memory_info(self) -> Dict[str, Any]:
        info = {
            "total_bytes": 0, "available_bytes": 0, "used_bytes": 0, "free_bytes": 0,
            "cached_bytes": 0, "buffers_bytes": 0, "swap_total_bytes": 0, "swap_free_bytes": 0,
            "swap_used_bytes": 0, "usage_percent": 0, "swap_usage_percent": 0
        }
        
        meminfo = self.read_proc_file("meminfo")
        if meminfo:
            mem_data = {}
            for line in meminfo.split("\n"):
                if ":" in line:
                    key, value = line.split(":")
                    try:
                        mem_data[key] = int(value.strip().replace(" kB", "")) * 1024
                    except ValueError:
                        pass
            
            info["total_bytes"] = mem_data.get("MemTotal", 0)
            info["free_bytes"] = mem_data.get("MemFree", 0)
            info["available_bytes"] = mem_data.get("MemAvailable", 0)
            info["cached_bytes"] = mem_data.get("Cached", 0)
            info["buffers_bytes"] = mem_data.get("Buffers", 0)
            info["swap_total_bytes"] = mem_data.get("SwapTotal", 0)
            info["swap_free_bytes"] = mem_data.get("SwapFree", 0)
            
            info["used_bytes"] = info["total_bytes"] - info["available_bytes"]
            info["swap_used_bytes"] = info["swap_total_bytes"] - info["swap_free_bytes"]
            
            if info["total_bytes"] > 0:
                info["usage_percent"] = (info["used_bytes"] / info["total_bytes"]) * 100
            if info["swap_total_bytes"] > 0:
                info["swap_usage_percent"] = (info["swap_used_bytes"] / info["swap_total_bytes"]) * 100
                
        return info
    
    def get_gpu_info(self) -> List[Dict[str, Any]]:
        gpus = []
        try:
            result = subprocess.run(
                ["nvidia-smi", "--query-gpu=index,name,temperature.gpu,memory.used,memory.total,utilization.gpu,power.draw,power.limit", 
                 "--format=csv,noheader,nounits"],
                capture_output=True, text=True, timeout=1
            )
            if result.returncode == 0:
                for line in result.stdout.strip().split("\n"):
                    parts = [p.strip() for p in line.split(",")]
                    if len(parts) >= 8:
                        gpus.append({
                            "index": int(parts[0]),
                            "name": parts[1],
                            "temperature_c": float(parts[2]) if parts[2] != "[N/A]" else 0,
                            "memory_used_mb": float(parts[3]) if parts[3] != "[N/A]" else 0,
                            "memory_total_mb": float(parts[4]) if parts[4] != "[N/A]" else 0,
                            "utilization_percent": float(parts[5]) if parts[5] != "[N/A]" else 0,
                            "power_draw_w": float(parts[6]) if parts[6] != "[N/A]" else 0,
                        })
        except Exception:
            pass
        return gpus

    def collect_all_metrics(self) -> Dict[str, Any]:
        return {
            "timestamp": datetime.now().isoformat(),
            "hostname": self.hostname,
            "cpu": self.get_cpu_info(),
            "memory": self.get_memory_info(),
            "gpu": self.get_gpu_info(),
            # Simplified for performance - add full disk/net if needed later
        }

# --- FastAPI App ---
@asynccontextmanager
async def lifespan(app: FastAPI):
    # Startup: Initialize psutil
    if psutil:
        psutil.cpu_percent() # First call returns 0
    yield
    # Shutdown logic if needed

app = FastAPI(lifespan=lifespan)
collector = SystemMetricsCollector()

@app.get("/metrics")
async def get_metrics():
    return JSONResponse(content=collector.collect_all_metrics())

@app.websocket("/ws/metrics")
async def websocket_endpoint(websocket: WebSocket):
    await websocket.accept()
    try:
        while True:
            data = collector.collect_all_metrics()
            await websocket.send_json(data)
            await asyncio.sleep(1) # 1Hz update rate
    except WebSocketDisconnect:
        print("Client disconnected")
    except Exception as e:
        print(f"WebSocket error: {e}")
        try:
            await websocket.close()
        except:
            pass

if __name__ == "__main__":
    # Remove existing socket if it exists
    if os.path.exists(UDS_PATH):
        os.remove(UDS_PATH)
        
    print(f"Starting Mission Control Daemon on unix:{UDS_PATH}")
    uvicorn.run(app, uds=UDS_PATH, loop="uvloop")
