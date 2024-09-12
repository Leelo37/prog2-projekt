import requests

projects = requests.get("http://127.0.0.1:7878/project").json()
for j in projects:
    if j["name"] == "Lucija & Lev":
        url = "http://" + j["ip"] + ":" + str(j["port"]) + "/sequence"
        print(url)
        seqs = requests.get(url).json()
        assert "Arithmetic" in [j["name"] for j in seqs]
<<<<<<< HEAD
        k = 0.9
        z = 1
        # for j in range(2):
        #     body = {
        #         "range": {
        #             "from": j * 100,
        #             "to": (j + 1) * 100,
        #             "step": 1,
        #         },
        #         "parameters": [z, k],
        #          "sequences": [
        #         ],
        #     }
        #     r = requests.post(url + "/Arithmetic", json=body)
        #     # print(r)
        #     print(r.json())
        for i in range(10):
=======
        k = 10
        z = 0
        for j in range(100):
>>>>>>> b45b57a47e504cf11a2fbe746ec7342a0c7e144a
            body = {
                "range": {
                    "from": i * 100,
                    "to": (i + 1) * 100,
                    "step": 1,
                },
                "parameters": [],
                "sequences": [{"name": "Geometric", "parameters": [z, k], "sequences": []}]
            }
<<<<<<< HEAD
            r = requests.post(url + "/Smoothed", json=body)
=======
            r = requests.post(url + "/Arithmetic", json=body)
            # print(r)
>>>>>>> b45b57a47e504cf11a2fbe746ec7342a0c7e144a
            print(r.json())
        break
else:
    print("Lucija & Lev not found")
    exit(1)
    exit(1)