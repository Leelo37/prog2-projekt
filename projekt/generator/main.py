import requests

projects = requests.get("http://127.0.0.1:7878/project").json()
for j in projects:
    if j["name"] == "Lucija & Lev":
        url = "http://" + j["ip"] + ":" + str(j["port"]) + "/sequence"
        print(url)
        seqs = requests.get(url).json()
        k = 0.9
        z = 1
        for j in range(2):
            body = {
                "range": {
                    "from": j * 100,
                    "to": (j + 1) * 100,
                    "step": 1,
                },
                "parameters": [z, k],
                 "sequences": [
                ],
            }
            r = requests.post(url + "/Arithmetc", json=body)
            print(r.json())
        for i in range(3):
            body = {
                "range": {
                    "from": i * 100,
                    "to": (i + 1) * 100,
                    "step": 1,
                },
                "parameters": [],
                "sequences": [{"name": "Arithmetic", "parameters": [z, k], "sequences": []}]
            }
            r = requests.post(url + "/Smoothed", json=body)
            print(r.json())
        break
    else:
        print("Lucija & Lev not found")
        exit(1)
        exit(1)
