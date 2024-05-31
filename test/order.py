import requests as re
import json
import config
from config import session_requests

data = {
  "base": "string",
  "quote": "string",
  "type": "string",
  "price": 0,
  "quantity": 0
}
res = session_requests.post(config.url + 'order', json=data)
print(res)
print(json.loads(res.text))

res = session_requests.get(config.url + f'order?id=&st=0&len=1&filter=')
print(res)
print(json.loads(res.text))