import requests

login_url = "https://users.premierleague.com/accounts/login/"
payload = {
    "login": "polortiz4@hotmail.com",
    "password": "password",
    "redirect_uri": "https://fantasy.premierleague.com/",
    "app": "plfpl-web",
}
s = requests.session()

s.post(login_url, data=payload)

transfers = []
transfers.append(
    # {"element_in": 233, "element_out": 272, "purchase_price": 125, "selling_price": 77}
    {"element_in": 272, "element_out": 233, "purchase_price": 77, "selling_price": 125}
)
payload = {
    "confirmed": True,
    "entry": 7597109,
    "event": 4,
    "transfers": transfers,
    "wildcard": False,
    "freehit": False,
}

headers = {
    "Content-Type": "application/json; charset=UTF-8",
    "X-Requested-With": "XMLHttpRequest",
    "Referer": "https://fantasy.premierleague.com/a/squad/transfers",
}

import json
r = s.post("https://fantasy.premierleague.com/api/transfers/", data=json.dumps(payload), headers=headers)

print(json.dumps(payload))
print(payload)
print(r.text)
