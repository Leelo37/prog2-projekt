import requests

projects = requests.get("http://127.0.0.1:7878/project").json()
print(projects)
for j in projects:
    if j["name"] == "Matija & Filip":
        url = "http://" + j["ip"] + ":" + str(j["port"]) + "/sequence"
        print(url)
        seqs = requests.get(url).json()
        print(seqs)
        assert "Geometric" in [j["name"] for j in seqs]
        k = 2
        z = 1
        for j in range(10):
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
            r = requests.post(url + "/Geometric", json=body)
            # print(r)
            print(r.json())
        break
else:
    print("Matija & Filip not found")
    exit(1)
    exit(1)
