import os
import glob
import re

rs_files = glob.glob('src/**/*.rs', recursive=True)
files_changed = 0

for file_path in rs_files:
    if "app_helpers.rs" in file_path:
        continue

    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    new_content = content.replace('ui.horizontal(|ui| {', 'crate::app_helpers::center_row(ui, |ui| {')
    new_content = new_content.replace('ui.horizontal( |ui| {', 'crate::app_helpers::center_row(ui, |ui| {')

    if new_content != content:
        files_changed += 1
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(new_content)
        print(f'Replaced in {file_path}')

print(f"Changed {files_changed} files")
