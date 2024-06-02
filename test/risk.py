import requests as re
import json
import config
from config import session_requests

data = {
  "risk_type": "buy",
  "on": True,
  "pnl": 0,
  "position": "BTC/USDT",
  "pid": 0
}
res = session_requests.post(config.url + 'risk', json=data)
print(res)
print(json.loads(res.text))

res = session_requests.get(config.url + 'risk')
print(res)
print(json.loads(res.text))