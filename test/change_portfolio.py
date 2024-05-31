import requests as re
import json
import config
from config import session_requests

data = {
  "name": "test6",
  "amount": 6000,
  "symbol": "USDT",
}

res = session_requests.put(config.url + 'portfolio', json=data)
print(res)
print(json.loads(res.text))