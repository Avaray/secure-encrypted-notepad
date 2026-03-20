import os
import glob

# Ensure we are in the project root
if not os.path.exists('src'):
    print("Cannot find src directory.")
    exit(1)

rs_files = glob.glob('src/**/*.rs', recursive=True)

total_replacements = 0
files_changed = 0

for file_path in rs_files:
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # We want to replace exactly `ui.horizontal(|ui| {` with `ui.horizontal_centered(|ui| {`
    # Warning: there could be multiple spaces after the pipe, or `horizontal` might be used as `horizontal_wrapped`.
    # Let's count them safely:
    new_content = content.replace('.horizontal(|ui|', '.horizontal_centered(|ui|')
    new_content = new_content.replace('.horizontal( |ui|', '.horizontal_centered(|ui|')
    
    if new_content != content:
        replacements = content.count('.horizontal(|ui|') + content.count('.horizontal( |ui|')
        total_replacements += replacements
        files_changed += 1
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(new_content)
        print(f"Updated {file_path} ({replacements} occurrences)")

print(f"Done! Changed {files_changed} files, {total_replacements} replacements total.")
