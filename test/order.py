import requests as re
import json
import config
from config import session_requests

data = {
  "base": "BTC",
  "quote": "USDT",
  "order_type": "buy",
  "price": "0",
  "quantity": "0",
  "portfolio_id": 4
}
res = session_requests.post(config.url + 'order', json=data)
print(res)
print(json.loads(res.text))

res = session_requests.get(config.url + f'order?id=4')
print(res)
print(json.loads(res.text))