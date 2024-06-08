import requests as re
import json
import config
from config import session_requests

data = {
  "name": "test6",
}

res = session_requests.delete(config.url + 'portfolio', params=data)
print(res)
print(json.loads(res.text))