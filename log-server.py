#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
TeamShell 日志服务器
接收前端发送的日志并写入文件系统
"""

from http.server import HTTPServer, BaseHTTPRequestHandler
import json
import os
from datetime import datetime
from urllib.parse import urlparse, parse_qs

class LogHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        parsed = urlparse(self.path)
        path = parsed.path

        if path == '/':
            self.send_response(200)
            self.send_header('Content-type', 'text/html; charset=utf-8')
            self.end_headers()
            with open('teamshell-terminal.html', 'rb') as f:
                self.wfile.write(f.read())
            return

        if path == '/api/logs':
            self.send_response(200)
            self.send_header('Content-type', 'application/json; charset=utf-8')
            self.end_headers()
            self.wfile.write(json.dumps({'status': 'logs received'}).encode('utf-8'))
            return

        if path.endswith('.html'):
            try:
                with open(path.lstrip('/'), 'rb') as f:
                    self.send_response(200)
                    self.send_header('Content-type', 'text/html; charset=utf-8')
                    self.end_headers()
                    self.wfile.write(f.read())
                    return
            except FileNotFoundError:
                pass

        self.send_response(404)
        self.end_headers()

    def do_POST(self):
        if self.path == '/api/logs':
            content_length = int(self.headers['Content-Length'])
            post_data = self.rfile.read(content_length)

            try:
                data = json.loads(post_data.decode('utf-8'))

                # 创建日志目录
                log_dir = os.path.join(os.path.dirname(__file__), 'logs')
                os.makedirs(log_dir, exist_ok=True)

                # 写入日志文件
                log_file = os.path.join(log_dir, f'logs_{datetime.now().strftime("%Y%m%d")}.txt')

                with open(log_file, 'a', encoding='utf-8') as f:
                    f.write(f"\n{'='*100}\n")
                    f.write(f"时间: {data.get('timestamp', '')}\n")
                    f.write(f"工位ID: {data.get('tabId', '')}\n")
                    f.write(f"工位名: {data.get('tabName', '')}\n")
                    f.write(f"类型: {data.get('type', '')}\n")
                    f.write(f"发送者: {data.get('sender', '')}\n")
                    f.write(f"发送者ID: {data.get('senderId', '')}\n")
                    f.write(f"内容: {data.get('content', '')}\n")
                    f.write(f"{'='*100}\n")

                print(f"✅ 日志已写入: {log_file}")

                self.send_response(200)
                self.send_header('Content-type', 'application/json; charset=utf-8')
                self.end_headers()
                self.wfile.write(json.dumps({'status': 'success'}).encode('utf-8'))
            except Exception as e:
                print(f"❌ 日志写入失败: {e}")
                self.send_response(500)
                self.send_header('Content-type', 'application/json; charset=utf-8')
                self.end_headers()
                self.wfile.write(json.dumps({'status': 'error', 'message': str(e)}).encode('utf-8'))
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        # 静默服务器日志
        pass

def run_server(port=9000):
    server = HTTPServer(('localhost', port), LogHandler)
    print(f"🚀 TeamShell 日志服务器运行在 http://localhost:{port}")
    print(f"📁 日志文件保存在: logs/")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n👋 服务器已停止")
        server.shutdown()

if __name__ == '__main__':
    run_server()