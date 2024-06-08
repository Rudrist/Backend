import requests as re
import json
import config
from config import session_requests

data = {
  "name": "admin",
  "password": "123456"
}

res = session_requests.post(config.url + 'auth/login', json=data)
print(res)
print(json.loads(res.text))
print(session_requests.cookies.get_dict())

res = session_requests.get(config.url + 'auth/user')
print(res)
print(json.loads(res.text))
print(session_requests.cookies.get_dict())  