import json
from typing import Any

data: dict[str,Any] = json.load(open("products.json",'r'))
counters: dict[str, int] = {}
for p in data["products"]:
    if not p.get("sku"):
        b = p["brand_slug"].upper().replace("-", "")[:4]
        counters[b] = counters.get(b, 0) + 1
        p["sku"] = f"{b}-{counters[b]:03d}"
json.dump(data, open("products.json", "w"), indent=2, ensure_ascii=False)
print(f"✓ {len(data['products'])} products, SKUs filled")