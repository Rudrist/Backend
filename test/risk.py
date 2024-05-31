import requests as re
import json
import config
from config import session_requests

data = {
  "risk_type": "string",
  "on": True,
  "pnl": 0,
  "position": "string",
  "pid": "3fa85f64-5717-4562-b3fc-2c963f66afa6"
}
res = session_requests.post(config.url + 'risk', json=data)
print(res)
print(json.loads(res.text))

res = session_requests.get(config.url + 'risk')
print(res)
print(json.loads(res.text))