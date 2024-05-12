import base64
import os
from typing import List

USER=os.getenv('USER')
RESOURCES_DIR=f"/Users/{USER}/Library/Application Support/Alfred 3/Alfred.alfredpreferences/resources"
DATA_PLIST=f"/Users/{USER}/Library/Application Support/Alfred 3/Alfred.alfredpreferences/preferences/features/websearch/prefs.plist"

def traverse_plist(path: str):
    entries = []
    with open(path, 'r') as f:
        lines = f.readlines()
        num_lines = len(lines)
        i = 5

        while i < num_lines:
            line = lines[i].strip()

            if line.startswith("<key>"):
                ky = line[5:-6]
                # print(f"[dbg][E][{i}][key] {ky}")
                i += 2  # After every key is a blank dict so skip that
                (entry, nw_i) = parse_entry(i, lines, ky)
                # print(f"\t[dbg][V][{nw_i}] ====> DONE {entry}")
                entries.append(make_entry(*entry))
                i = nw_i
            # else:
                # print(f"[dbg][E][{i}][skip] {line}")

            i += 1

    return entries

def parse_entry(i: int, lines: List[str], ky: str) -> (List[str], int):
    entry = ["","","",ky]

    while not lines[i].strip().startswith("</dict>"):
        line = lines[i].strip()

        if line.startswith("<key>"):
            ky = line[5:-6]
            # print(f"\t[dbg][V][{i}] {lines[i].strip()} -> {ky}")
            i += 1
            value = lines[i].strip()
            # print(f"\t[dbg][V][{i}] {lines[i].strip()}")
            if not value.startswith("<string"):
                i += 1
                continue
            value = value[8:-9]

            if ky == 'keyword':
                entry[1] = value
            elif ky == 'text':
                entry[0] = value
            elif ky == 'url':
                entry[2] = value.replace("{query}", "{0}", 1)

            i += 1
            continue

        i += 1
    return entry, i


def convert_resource_file_to_dataurl(ky: str) -> str:
    if ky == "":
        return ""
    try:
        filepath = f'{RESOURCES_DIR}/features.websearch.custom.{ky}.png'
        binary_fc = open(filepath, 'rb').read()
        base64_utf8_str = base64.b64encode(binary_fc).decode('utf-8')
        ext = filepath.split('.')[-1]
        return f'data:image/{ext};base64,{base64_utf8_str}'
    except:
        return ""

def make_entry(
    label: str,
    shortname: str,
    template: str,
    icon_key: str,
) -> str:
    return (
        f'[webq.searchers."{label}"]\n'
        f'label = "{label}"\n'
        f'shortname = "{shortname}"\n'
        f'template = "{template}"\n'
        f'icon = "{convert_resource_file_to_dataurl(icon_key)}"\n'
    )

print('\n')
for entry in traverse_plist(DATA_PLIST):
    print(entry)
