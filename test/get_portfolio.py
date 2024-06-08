import requests as re
import json
import config
from config import session_requests

res = session_requests.get(config.url + 'portfolio')
print(res)
print(json.loads(res.text))