#!/usr/bin/env python3
# script to remove hardcode dollar sign in frontend
import os
import re
import sys

def find_money_dollar_signs(root_dir="frontend/src"):
    results = []

    for root, dirs, files in os.walk(root_dir):
        dirs[:] = [d for d in dirs if d not in ('node_modules', 'dist', '.git')]
        for f in files:
            if not f.endswith(('.vue', '.js', '.ts', '.html')):
                continue
            filepath = os.path.join(root, f)
            relpath = os.path.relpath(filepath)
            with open(filepath, 'r', encoding='utf-8') as fh:
                for line_no, line in enumerate(fh, start=1):
                    # Check for money dollar sign:
                    # 1. $ followed by {{ (Vue template currency: {{ ... }})
                    # 2. $ followed by ${ (JS template literal currency: $${ ... })
                    # 3. $ followed by a digit or decimal: $10, $0.00
                    # 4. prefix="$" or 'USD ($)'
                    # 5. Any $ not part of ${...} or Vue built-in ($router, $route, $emit, $refs, $nextTick, $event)
                    
                    # JS template literal interpolation has a single '{' (not double '{{'):
                    # e.g. `${orderId}`, `${token}`.
                    # In contrast, `{{ foo }}` in Vue templates is a dollar sign before `{{ foo }}`!
                    # First, protect `$${` (template string with leading dollar) and `{{` (Vue template with leading dollar):
                    temp_line = re.sub(r'\$\$\{', 'MONEY_DOLLAR${', line)
                    temp_line = re.sub(r'\$\{\{', 'MONEY_DOLLAR{{', temp_line)
                    
                    # Remove true JS template literals ${...} where { is not followed by {
                    temp_line = re.sub(r'\$\{[^{][^}]*\}', '', temp_line)
                    
                    # Remove Vue built-in variables like $router, $route, $emit, etc.
                    temp_line = re.sub(r'\$(router|route|emit|refs|nextTick|event|slots|attrs)\b', '', temp_line)
                    
                    # If there's still a $ or MONEY_DOLLAR, it's a currency dollar sign!
                    if '$' in temp_line or 'MONEY_DOLLAR' in temp_line:
                        results.append((relpath, line_no, line.rstrip()))

    return results

if __name__ == '__main__':
    search_dir = sys.argv[1] if len(sys.argv) > 1 else "frontend/src"
    matches = find_money_dollar_signs(search_dir)
    print(f"Total lines found: {len(matches)}\n")
    for path, line_no, line in matches:
        print(f"{path}:{line_no}: {line}")
