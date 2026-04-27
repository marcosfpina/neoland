#!/usr/bin/env python3

"""
Advanced Performance Profiler for Dev Assistant Hub
Monitors system resources, application metrics, and provides detailed analysis
"""

import psutil
import requests
import time
import json
import os
import sys
import threading
import statistics
from datetime import datetime, timedelta
from collections import defaultdict, deque
import argparse

class PerformanceProfiler:
    def __init__(self, base_url="http://localhost:3000", duration=60, interval=1):
        self.base_url = base_url
        self.duration = duration
        self.interval = interval
        self.results_dir = "benchmarks/results"
        
        # Metrics storage
        self.metrics = {
            'system': defaultdict(list),
            'application': defaultdict(list),
            'network': defaultdict(list),
            'endpoints': defaultdict(lambda: defaultdict(list))
        }
        
        # Application process tracking
        self.app_processes = []
        self.monitoring = False
        
        # Ensure results directory exists
        os.makedirs(self.results_dir, exist_ok=True)
        
    def find_app_processes(self):
        """Find processes related to the Dev Assistant Hub application"""
        processes = []
        for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
            try:
                cmdline = ' '.join(proc.info['cmdline'] or [])
                if any(keyword in cmdline.lower() for keyword in ['next', 'node', 'dev-assistant']):
                    processes.append(proc)
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
        return processes
    
    def collect_system_metrics(self):
        """Collect system-wide performance metrics"""
        # CPU metrics
        cpu_percent = psutil.cpu_percent(interval=None)
        cpu_per_core = psutil.cpu_percent(interval=None, percpu=True)
        cpu_freq = psutil.cpu_freq()
        
        self.metrics['system']['cpu_percent'].append(cpu_percent)
        self.metrics['system']['cpu_cores'].append(cpu_per_core)
        if cpu_freq:
            self.metrics['system']['cpu_freq_mhz'].append(cpu_freq.current)
        
        # Memory metrics
        memory = psutil.virtual_memory()
        swap = psutil.swap_memory()
        
        self.metrics['system']['memory_percent'].append(memory.percent)
        self.metrics['system']['memory_used_gb'].append(memory.used / (1024**3))
        self.metrics['system']['memory_available_gb'].append(memory.available / (1024**3))
        self.metrics['system']['swap_percent'].append(swap.percent)
        
        # Disk I/O
        disk_io = psutil.disk_io_counters()
        if disk_io:
            self.metrics['system']['disk_read_mb'].append(disk_io.read_bytes / (1024**2))
            self.metrics['system']['disk_write_mb'].append(disk_io.write_bytes / (1024**2))
        
        # Network I/O
        network_io = psutil.net_io_counters()
        if network_io:
            self.metrics['system']['network_sent_mb'].append(network_io.bytes_sent / (1024**2))
            self.metrics['system']['network_recv_mb'].append(network_io.bytes_recv / (1024**2))
    
    def collect_application_metrics(self):
        """Collect application-specific metrics"""
        total_cpu = 0
        total_memory = 0
        total_threads = 0
        total_fds = 0
        
        current_processes = self.find_app_processes()
        
        for proc in current_processes:
            try:
                # CPU usage
                cpu_percent = proc.cpu_percent()
                total_cpu += cpu_percent
                
                # Memory usage
                memory_info = proc.memory_info()
                memory_mb = memory_info.rss / (1024**2)
                total_memory += memory_mb
                
                # Thread count
                total_threads += proc.num_threads()
                
                # File descriptors (Unix-like systems)
                if hasattr(proc, 'num_fds'):
                    total_fds += proc.num_fds()
                
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue
        
        self.metrics['application']['cpu_percent'].append(total_cpu)
        self.metrics['application']['memory_mb'].append(total_memory)
        self.metrics['application']['threads'].append(total_threads)
        self.metrics['application']['file_descriptors'].append(total_fds)
        self.metrics['application']['process_count'].append(len(current_processes))
    
    def test_endpoint_performance(self, endpoint, method='GET', payload=None):
        """Test individual endpoint performance"""
        url = f"{self.base_url}{endpoint}"
        
        try:
            start_time = time.time()
            
            if method.upper() == 'POST':
                response = requests.post(url, json=payload, timeout=10)
            else:
                response = requests.get(url, timeout=10)
            
            end_time = time.time()
            response_time = (end_time - start_time) * 1000  # Convert to milliseconds
            
            self.metrics['endpoints'][endpoint]['response_time_ms'].append(response_time)
            self.metrics['endpoints'][endpoint]['status_code'].append(response.status_code)
            self.metrics['endpoints'][endpoint]['response_size_bytes'].append(len(response.content))
            
            return {
                'success': True,
                'response_time': response_time,
                'status_code': response.status_code,
                'response_size': len(response.content)
            }
            
        except requests.RequestException as e:
            self.metrics['endpoints'][endpoint]['errors'].append(str(e))
            return {
                'success': False,
                'error': str(e)
            }
    
    def stress_test_endpoints(self):
        """Continuously stress test various endpoints"""
        endpoints = [
            {'path': '/api/health', 'method': 'GET'},
            {'path': '/api/code-analyzer', 'method': 'POST', 'payload': {'code': 'console.log("test");'}},
            {'path': '/api/llm-playground', 'method': 'GET'},
            {'path': '/', 'method': 'GET'}
        ]
        
        while self.monitoring:
            for endpoint_config in endpoints:
                if not self.monitoring:
                    break
                    
                self.test_endpoint_performance(
                    endpoint_config['path'],
                    endpoint_config.get('method', 'GET'),
                    endpoint_config.get('payload')
                )
                
                time.sleep(0.5)  # Small delay between endpoint tests
    
    def monitor_performance(self):
        """Main monitoring loop"""
        print(f"🔍 Starting performance monitoring for {self.duration} seconds...")
        print(f"📊 Monitoring interval: {self.interval} seconds")
        print(f"🎯 Target URL: {self.base_url}")
        
        self.monitoring = True
        start_time = time.time()
        
        # Start endpoint stress testing in a separate thread
        stress_thread = threading.Thread(target=self.stress_test_endpoints)
        stress_thread.daemon = True
        stress_thread.start()
        
        # Main monitoring loop
        while time.time() - start_time < self.duration:
            loop_start = time.time()
            
            # Collect metrics
            self.collect_system_metrics()
            self.collect_application_metrics()
            
            # Progress indicator
            elapsed = time.time() - start_time
            remaining = self.duration - elapsed
            progress = (elapsed / self.duration) * 100
            
            print(f"\r⏱️  Progress: {progress:.1f}% | "
                  f"Elapsed: {elapsed:.1f}s | "
                  f"Remaining: {remaining:.1f}s | "
                  f"CPU: {self.metrics['system']['cpu_percent'][-1]:.1f}% | "
                  f"Memory: {self.metrics['application']['memory_mb'][-1]:.1f}MB", 
                  end='', flush=True)
            
            # Sleep for the remainder of the interval
            sleep_time = self.interval - (time.time() - loop_start)
            if sleep_time > 0:
                time.sleep(sleep_time)
        
        self.monitoring = False
        print("\n✅ Performance monitoring completed!")
    
    def calculate_statistics(self, data_list):
        """Calculate statistical metrics for a data series"""
        if not data_list:
            return {}
        
        return {
            'min': min(data_list),
            'max': max(data_list),
            'mean': statistics.mean(data_list),
            'median': statistics.median(data_list),
            'std_dev': statistics.stdev(data_list) if len(data_list) > 1 else 0,
            'p95': statistics.quantiles(data_list, n=20)[18] if len(data_list) >= 20 else max(data_list),
            'p99': statistics.quantiles(data_list, n=100)[98] if len(data_list) >= 100 else max(data_list)
        }
    
    def generate_report(self):
        """Generate comprehensive performance report"""
        timestamp = datetime.now().isoformat()
        
        # Calculate statistics for all metrics
        report = {
            'timestamp': timestamp,
            'configuration': {
                'base_url': self.base_url,
                'duration_seconds': self.duration,
                'monitoring_interval': self.interval
            },
            'system_performance': {},
            'application_performance': {},
            'endpoint_performance': {},
            'summary': {}
        }
        
        # System performance statistics
        for metric, values in self.metrics['system'].items():
            if values:
                report['system_performance'][metric] = self.calculate_statistics(values)
        
        # Application performance statistics
        for metric, values in self.metrics['application'].items():
            if values:
                report['application_performance'][metric] = self.calculate_statistics(values)
        
        # Endpoint performance statistics
        for endpoint, metrics in self.metrics['endpoints'].items():
            report['endpoint_performance'][endpoint] = {}
            for metric, values in metrics.items():
                if values and metric != 'errors':
                    report['endpoint_performance'][endpoint][metric] = self.calculate_statistics(values)
                elif metric == 'errors':
                    report['endpoint_performance'][endpoint]['error_count'] = len(values)
                    report['endpoint_performance'][endpoint]['errors'] = values[:10]  # First 10 errors
        
        # Generate summary
        cpu_avg = statistics.mean(self.metrics['system']['cpu_percent']) if self.metrics['system']['cpu_percent'] else 0
        memory_avg = statistics.mean(self.metrics['application']['memory_mb']) if self.metrics['application']['memory_mb'] else 0
        
        # Calculate overall endpoint performance
        all_response_times = []
        total_requests = 0
        total_errors = 0
        
        for endpoint_metrics in self.metrics['endpoints'].values():
            if 'response_time_ms' in endpoint_metrics:
                all_response_times.extend(endpoint_metrics['response_time_ms'])
                total_requests += len(endpoint_metrics['response_time_ms'])
            if 'errors' in endpoint_metrics:
                total_errors += len(endpoint_metrics['errors'])
        
        report['summary'] = {
            'average_cpu_percent': round(cpu_avg, 2),
            'average_memory_mb': round(memory_avg, 2),
            'total_requests': total_requests,
            'total_errors': total_errors,
            'error_rate_percent': round((total_errors / max(total_requests, 1)) * 100, 2),
            'average_response_time_ms': round(statistics.mean(all_response_times), 2) if all_response_times else 0,
            'p95_response_time_ms': round(statistics.quantiles(all_response_times, n=20)[18], 2) if len(all_response_times) >= 20 else 0
        }
        
        # Save report
        filename = os.path.join(self.results_dir, f"performance-profile-{timestamp.replace(':', '-')}.json")
        with open(filename, 'w') as f:
            json.dump(report, f, indent=2)
        
        return report, filename
    
    def print_summary(self, report):
        """Print a human-readable summary of the performance report"""
        print("\n" + "="*60)
        print("📊 PERFORMANCE PROFILING SUMMARY")
        print("="*60)
        
        summary = report['summary']
        print(f"⏱️  Duration: {self.duration} seconds")
        print(f"🖥️  Average CPU Usage: {summary['average_cpu_percent']}%")
        print(f"💾 Average Memory Usage: {summary['average_memory_mb']:.1f} MB")
        print(f"🌐 Total Requests: {summary['total_requests']}")
        print(f"❌ Total Errors: {summary['total_errors']}")
        print(f"📈 Error Rate: {summary['error_rate_percent']}%")
        print(f"⚡ Average Response Time: {summary['average_response_time_ms']:.2f} ms")
        print(f"🎯 P95 Response Time: {summary['p95_response_time_ms']:.2f} ms")
        
        print("\n📋 ENDPOINT PERFORMANCE:")
        for endpoint, metrics in report['endpoint_performance'].items():
            if 'response_time_ms' in metrics:
                avg_time = metrics['response_time_ms']['mean']
                p95_time = metrics['response_time_ms']['p95']
                print(f"  {endpoint}: {avg_time:.2f}ms avg, {p95_time:.2f}ms P95")
        
        print("\n🔧 SYSTEM RESOURCES:")
        if 'cpu_percent' in report['system_performance']:
            cpu_stats = report['system_performance']['cpu_percent']
            print(f"  CPU: {cpu_stats['mean']:.1f}% avg, {cpu_stats['max']:.1f}% peak")
        
        if 'memory_used_gb' in report['system_performance']:
            mem_stats = report['system_performance']['memory_used_gb']
            print(f"  Memory: {mem_stats['mean']:.2f}GB avg, {mem_stats['max']:.2f}GB peak")

def main():
    parser = argparse.ArgumentParser(description='Performance Profiler for Dev Assistant Hub')
    parser.add_argument('--url', default='http://localhost:3000', help='Base URL of the application')
    parser.add_argument('--duration', type=int, default=60, help='Monitoring duration in seconds')
    parser.add_argument('--interval', type=float, default=1.0, help='Monitoring interval in seconds')
    
    args = parser.parse_args()
    
    profiler = PerformanceProfiler(
        base_url=args.url,
        duration=args.duration,
        interval=args.interval
    )
    
    try:
        # Run the performance monitoring
        profiler.monitor_performance()
        
        # Generate and save report
        report, filename = profiler.generate_report()
        
        # Print summary
        profiler.print_summary(report)
        
        print(f"\n💾 Detailed report saved to: {filename}")
        
    except KeyboardInterrupt:
        print("\n⚠️  Monitoring interrupted by user")
        profiler.monitoring = False
    except Exception as e:
        print(f"\n❌ Error during profiling: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
