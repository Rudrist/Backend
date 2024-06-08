import requests as re
import json
import config
from config import session_requests

data = {
  "name": "test6",
  "position": [
        "BTC/USDT"
    ]
}

res = session_requests.post(config.url + 'portfolio', json=data)
print(res)
print(json.loads(res.text))