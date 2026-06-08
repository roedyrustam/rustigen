import re

content = open('temp_yt.html', encoding='utf-8').read()

meta_tags = re.findall(r'<meta[^>]+>', content)
for meta in meta_tags:
    if 'description' in meta.lower() or 'og:' in meta.lower() or 'subscriber' in meta.lower():
        print(meta.encode('ascii', errors='replace').decode('ascii'))
