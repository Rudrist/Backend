import requests as re
import json
import config
from config import session_requests

data = {
  "name": "admin",
  "password": "123456",
  "user_type": 0,
  "email": "admin@localhost",
}

res = session_requests.post(config.url + 'auth/singup', json=data)
print(res)
print(json.loads(res.text))