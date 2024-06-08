import requests as re
import json
import config
from config import session_requests

data = {
  "current_password": "654321",
  "new_password": "123456"
}

res = session_requests.post(config.url + 'auth/reset', json=data)
print(res)
print(json.loads(res.text))
print(session_requests.cookies.get_dict())