import requests as re
import json
import config
from config import session_requests

res = session_requests.post(config.url + 'auth/logout')
print(res)