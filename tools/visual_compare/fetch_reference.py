"""Fetch the analysis-only reference; never used by the application."""
import hashlib
import json
from pathlib import Path
import urllib.request

root = Path(__file__).parent
metadata = json.loads((root / 'measurements.json').read_text())
request = urllib.request.Request(metadata['image_url'], headers={'User-Agent': 'HP67VisualStudy/1.0'})
data = urllib.request.urlopen(request, timeout=30).read()
if hashlib.sha256(data).hexdigest() != metadata['sha256']:
    raise SystemExit('Reference changed; review measurements before accepting a replacement.')
(root / 'reference.jpg').write_bytes(data)
print(metadata['source'])
