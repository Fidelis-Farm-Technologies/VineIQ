import requests
import time

# url = "https://api.yosmart.com/open/yolink/token"
url = "http://localhost/open/yolink/token"
ua_id = "ua_F08AC7C7C5C44C2D81B84C50ACAB674F"
sec_id = "sec_v1_0rjd714qf+CjAufbiBVaJg=="

data = {
    'grant_type': 'client_credentials'
}


response = requests.post(
    url,
    data=data,
    auth=(ua_id, sec_id)
)
print(response.text)

if response.status_code != 200:
    print(("failed to get access token: {}").format(response.status_code),file=sys.stderr)
else:
    print (response.content)


